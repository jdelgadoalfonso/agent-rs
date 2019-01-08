#![feature(panic_info_message)]
#![feature(nll)]
#![feature(await_macro, async_await, futures_api)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate influx_db_client;
#[macro_use]
extern crate log;

mod config;
mod panic;
mod parser;
mod rabbit;
mod service;
mod tsdb {
    pub mod push;
}

use crate::config::config;
use crate::parser::{parser, CHTHeader};
use crate::rabbit::connect_to_rabbit_tls;
use crate::service::index;
use crate::tsdb::push::push;
use crate::panic::set_panic_hook;

use actix::{Actor, Handler, Message, SyncArbiter, SyncContext};
use actix_web::{http, middleware::cors::Cors, server, App};
use failure::Error;
use futures::{future, stream::Stream, Future};
use lapin_futures::{
    channel::{BasicConsumeOptions, ConfirmSelectOptions, QueueDeclareOptions},
    types::FieldTable,
};
use nom::HexDisplay;
use rustls::{
    internal::pemfile::{certs, rsa_private_keys},
    NoClientAuth, ServerConfig,
};

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

    let fut = connect_to_rabbit_tls(&configuration, |_| ())
        .and_then(|(client, _)| {
            client
                .create_confirm_channel(ConfirmSelectOptions::default())
                .map_err(Error::from)
        })
        .and_then(|channel| {
            info!("created channel with id: {}", channel.id);
            channel
                .queue_declare("hello", QueueDeclareOptions::default(), FieldTable::new())
                .map_err(Error::from)
                .map(move |queue| (channel, queue))
        })
        .and_then(move |(channel, queue)| {
            info!("channel {} declared queue {}", channel.id, "hello");
            channel
                .basic_consume(
                    &queue,
                    "my_consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
                )
                .map_err(Error::from)
                .map(move |stream| (channel, stream))
        })
        .and_then(move |(channel, stream)| {
            info!("got consumer stream");
            let addr = SyncArbiter::start(2, || Summator);

            stream
                .for_each(move |message| {
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
                        }
                        _ => {
                            error!("error or incomplete");
                            error!("cannot parse header");
                        }
                    }
                    channel.basic_ack(message.delivery_tag, false)
                })
                .map_err(Error::from)
        });

    // load ssl keys
    let config = load_ssl();

    server::new(move || {
        App::new().configure(|app| {
            Cors::for_app(app) // <- Construct CORS middleware builder
                .allowed_origin("http://localhost:4200")
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .resource("/", |r| r.h(index))
                .register()
        })
    })
    .workers(2)
    .bind_with("0.0.0.0:8443", move || {
        server::RustlsAcceptor::with_flags(
            config.clone(),
            server::ServerFlags::HTTP1 | server::ServerFlags::HTTP2,
        )
    })
    .expect("Can not bind to 0.0.0.0:8443")
    .start();

    actix::spawn(fut.then(|res| {
        match res {
            Ok(_) => info!("Ok"),
            Err(e) => error!("{:?}", e),
        }
        actix::System::current().stop();
        future::result(Ok(()))
    }));

    let _ = sys.run();
}
