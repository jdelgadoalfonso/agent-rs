#![feature(panic_info_message)]

extern crate abstract_ns;
extern crate actix;
extern crate actix_web;
extern crate amq_protocol;
#[macro_use]
extern crate bitflags;
extern crate chrono;
extern crate config as ext_config;
extern crate elastic;
#[macro_use]
extern crate elastic_derive;
extern crate env_logger;
extern crate futures;
extern crate lapin_futures as lapin;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate ns_dns_tokio;
extern crate rustls;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tls_api;
extern crate tls_api_rustls;
extern crate tokio_core;
extern crate tokio_tls_api;
extern crate tokio_io;


mod config;
mod parser;
mod push;
mod rabbit;
mod service;


use actix::{Arbiter, msgs};

use actix_web::{server, App};

use amq_protocol::uri::{AMQPQueryString, AMQPUserInfo};

use config::{client_config_with_root_ca, config};

use futures::{future, Future, Stream};

use lapin::{
    types::FieldTable,
    channel::{
        BasicConsumeOptions, ConfirmSelectOptions, QueueDeclareOptions
    }
};

use nom::{HexDisplay, IResult};

use parser::parser;

use rabbit::{connect_stream, open_tcp_stream};

use service::index;

use std::panic;

use tls_api::TlsConnectorBuilder;


fn main() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(message) = panic_info.message() {
            error!("panic: {}", message);
        } else if let Some(payload) = panic_info.payload()
                                      .downcast_ref::<&'static str>() {
            error!("panic: {}", payload);
        } else {
            error!("panic");
        }
    }));

    let sys = actix::System::new("guide");
    let configuration = config();
    let vhost = configuration.vhost.clone();
    let userinfo = AMQPUserInfo {
        username: configuration.username.clone(),
        password: configuration.password.clone(),
    };
    let query = AMQPQueryString {
        frame_max: None,
        heartbeat: None,
    };

    drop(env_logger::init());

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
    }).and_then(|client| {
        client.create_confirm_channel(ConfirmSelectOptions::default())
    }).and_then(|channel| {
        let id = channel.id;
        info!("created channel with id: {}", id);

        let ch = channel.clone();
        channel.queue_declare(
            "hello", &QueueDeclareOptions::default(), &FieldTable::new()
        )
        .and_then(move |_| {
            info!("channel {} declared queue {}", id, "hello");

            // basic_consume returns a future of a message
            // stream. Any time a message arrives for this consumer,
            // the for_each method would be called
            channel.basic_consume(
                "hello", "my_consumer", &BasicConsumeOptions::default(),
                &FieldTable::new()
            )
        })
        .and_then(|stream| {
            info!("got consumer stream");

            stream.for_each(move |message| {
                debug!("got message: {:?}", message);
                info!("raw message:\n{}", &*message.data.to_hex(16));
                let header = parser(&message.data[..]);

                match header {
                    IResult::Done(i, o) => {
                        info!("parsed: {:?}\n", o);
                        if i.len() > 0 {
                            info!("remaining:\n{}", &i[..].to_hex(16));
                        }
                    },
                    _  => {
                        error!("error or incomplete");
                        error!("cannot parse header");
                    }
                }
                ch.basic_ack(message.delivery_tag);
                Ok(())
            })
        })
    });

    server::new(| | App::new().resource("/", |r| r.h(index)))
    .bind("0.0.0.0:8080").expect("Can not bind to 0.0.0.0:8080")
    .start();

    sys.handle().spawn(fut.then(|res| {
        match res {
            Ok(_) => info!("Ok"),
            Err(e) => error!("{:?}", e),
        }

        let _ = Arbiter::system().send(msgs::SystemExit(0));
        future::result(Ok(()))
    }));

    let _ = sys.run();
}
