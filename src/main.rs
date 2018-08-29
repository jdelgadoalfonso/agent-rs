#![feature(panic_info_message)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate influx_db_client;
#[macro_use]
extern crate log;

mod config;
mod parser;
mod tsdb {
    pub mod push;
}
mod rabbit;
mod service;

use crate::config::{client_config_with_root_ca, config};
use crate::parser::{CHTHeader, parser};
use crate::tsdb::push::push;
use crate::rabbit::{connect_stream, open_tcp_stream};
use crate::service::index;

use actix::{Actor, Arbiter, SyncContext, Handler, Message, SyncArbiter};
use actix_web::{http, server, App, middleware::cors::Cors};
use amq_protocol::uri::{AMQPQueryString, AMQPUserInfo};
use futures::{future, Future, stream::Stream};
use lapin_futures::{
    types::FieldTable,
    channel::{
        BasicConsumeOptions, ConfirmSelectOptions, QueueDeclareOptions
    }
};
use nom::HexDisplay;
use rustls::{
    NoClientAuth, ServerConfig,
    internal::pemfile::{certs, rsa_private_keys}
};
use std::panic;
use tls_api::TlsConnectorBuilder;


impl Message for CHTHeader {
    type Result = ();
}

// Actor definition
struct Summator;

impl Actor for Summator {
    type Context = SyncContext<Self>;
}

// now we need to define `MessageHandler` for the `Sum` message.
impl Handler<CHTHeader> for Summator {
    type Result = (); // <- Message response type

    fn handle(&mut self, msg: CHTHeader, _ctx: &mut Self::Context) {
        info!("Handling message...");
        if let Some(ls) = &msg.data {
            for s in ls {
                info!("Storing StatSta {:?}\n", s);
                push(&msg, &s);
            }
        }
    }
}

fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(message) = panic_info.message() {
            error!("panic: {}", message);
        } else if let Some(payload) = panic_info
        .payload().downcast_ref::<&'static str>() {
            error!("panic: {}", payload);
        } else {
            error!("panic");
        }
    }));
}

fn load_ssl() -> ServerConfig {
    use std::io::BufReader;

    const CERT: &'static [u8] = include_bytes!("../resources/cert.pem");
    const KEY: &'static [u8] = include_bytes!("../resources/key.pem");

    let mut cert = BufReader::new(CERT);
    let mut key = BufReader::new(KEY);

    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_chain = certs(&mut cert).unwrap();
    let mut keys = rsa_private_keys(&mut key).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    config
}

fn main() {
    set_panic_hook();
    drop(env_logger::init());

    let sys = actix::System::new("agent-rs");
    let configuration = config();
    let vhost = configuration.vhost.clone();
    let userinfo = AMQPUserInfo {
        username: configuration.username.clone(),
        password: configuration.password.clone(),
    };
    let query = AMQPQueryString {
        frame_max: None,
        heartbeat: Some(20),
    };

    let fut = open_tcp_stream(&configuration.host, configuration.port)
    .join({
        let tls_connector_builder = tls_api_rustls::TlsConnectorBuilder(
            client_config_with_root_ca(&configuration.cafile)
        );
        let tls_connector = tls_connector_builder.build().map_err(From::from);
        futures::future::result(tls_connector)
    }).and_then(move |(stream, connector)| {
        tokio_tls_api::connect_async(&connector, &configuration.host, stream)
        .map_err(From::from).map(Box::new)
    }).and_then(move |stream| {
        connect_stream(stream, userinfo, vhost, &query, |_| ())
    }).and_then(|(client, _)| {
        client.create_confirm_channel(ConfirmSelectOptions::default())
    }).and_then(|channel| {
        info!("created channel with id: {}", channel.id);
        channel.queue_declare(
            "hello", QueueDeclareOptions::default(), FieldTable::new()
        ).map(move |queue| (channel, queue))
    }).and_then(move |(channel, queue)| {
        info!("channel {} declared queue {}", channel.id, "hello");
        channel.basic_consume(&queue, "my_consumer",
            BasicConsumeOptions::default(), FieldTable::new()
        ).map(move |stream| (channel, stream))
    }).and_then(move |(channel, stream)| {
        info!("got consumer stream");
        let addr = SyncArbiter::start(2, || Summator);

        stream.for_each(move |message| {
            debug!("got message: {:?}", message);
            info!("raw message:\n{}", &*message.data.to_hex(16));
            let header = parser(&message.data[..]);

            match header {
                Ok((i, o)) => {
                    if i.len() > 0 {
                        info!("remaining:\n{}", &i[..].to_hex(16));
                    }
                    info!("Sending message to SyncActor");
                    addr.do_send(o);
                    //Arbiter::spawn(res.then(|_| future::result(Ok(()))));
                },
                _  => {
                    error!("error or incomplete");
                    error!("cannot parse header");
                }
            }
            channel.basic_ack(message.delivery_tag, false)
        })
    });

    // load ssl keys
    let config = load_ssl();
    let acceptor = server::RustlsAcceptor::new(config);

    server::new(move || App::new().configure(|app| {
        Cors::for_app(app) // <- Construct CORS middleware builder
        .allowed_origin("http://localhost:4200")
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
        .resource("/", |r| r.h(index))
        .register()
    }))
    .workers(2)
    .bind_with("0.0.0.0:8443", acceptor)
    .expect("Can not bind to 0.0.0.0:8443")
    .start();

    Arbiter::spawn(fut.then(|res| {
        match res {
            Ok(_) => info!("Ok"),
            Err(e) => error!("{:?}", e),
        }

        actix::System::current().stop();
        future::result(Ok(()))
    }));

    let _ = sys.run();
}
