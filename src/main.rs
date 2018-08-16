#![feature(panic_info_message, rust_2018_preview)]

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

use actix::{Actor, Arbiter, Context, Handler, Message};
use actix_web::{server, App};
use amq_protocol::uri::{AMQPQueryString, AMQPUserInfo};
use futures::{future, Future, stream::Stream};
use lapin_futures::{
    types::FieldTable,
    channel::{
        BasicConsumeOptions, ConfirmSelectOptions, QueueDeclareOptions
    }
};
use nom::HexDisplay;
use std::panic;
use tls_api::TlsConnectorBuilder;


impl Message for CHTHeader {
    type Result = ();
}

// Actor definition
struct Summator;

impl Actor for Summator {
    type Context = Context<Self>;
}

// now we need to define `MessageHandler` for the `Sum` message.
impl Handler<CHTHeader> for Summator {
    type Result = (); // <- Message response type

    fn handle(&mut self, msg: CHTHeader, _ctx: &mut Context<Self>) {
        if let Some(ls) = msg.data {
            for s in ls {
                info!("Storing StatSta {:?}\n", s);
                push(s);
            }
        }
    }
}


fn main() {
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

    drop(env_logger::init());

    let sys = actix::System::new("guide");
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
        let addr = Arbiter::start(move |_| Summator);

        stream.for_each(move |message| {
            debug!("got message: {:?}", message);
            info!("raw message:\n{}", &*message.data.to_hex(16));
            let header = parser(&message.data[..]);

            match header {
                Ok((i, o)) => {
                    if i.len() > 0 {
                        info!("remaining:\n{}", &i[..].to_hex(16));
                    }
                    let res = addr.send(o);
                    Arbiter::spawn(res.then(|_| future::result(Ok(()))));
                },
                _  => {
                    error!("error or incomplete");
                    error!("cannot parse header");
                }
            }
            channel.basic_ack(message.delivery_tag, false)
        })
    });

    server::new(| | App::new().resource("/", |r| r.h(index)))
    .bind("0.0.0.0:8080").expect("Can not bind to 0.0.0.0:8080")
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
