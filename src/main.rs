#![feature(panic_info_message)]
#![feature(nll)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate influx_db_client;
#[macro_use]
extern crate log;
#[macro_use]
extern crate machine;
#[macro_use]
extern crate strum_macros;

mod config;
mod panic;
mod parser;
mod rabbit;
mod service;
mod tsdb {
    pub mod push;
}

use crate::config::config;
use crate::panic::set_panic_hook;
use crate::parser::{parser, CHTHeader};
use crate::rabbit::{
    //config_rabbit_connection,
    header_parser::parse_header,
};
use crate::service::index;
use crate::tsdb::push::push;

use actix::{Actor, Handler, Message, SyncArbiter, SyncContext};
use actix_cors::Cors;
use actix_web::{http, HttpServer, App, HttpRequest, HttpResponse, web};
use amq_protocol::{tcp::TcpStream, uri::{AMQPUri, AMQPScheme}};
use bytes::Bytes;
use failure::Error;
use futures::{future, Future, Async, Poll, Stream};
use lapin_futures::{
    {Client, ConnectionProperties},
    options::{BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable
};
use nom::HexDisplay;
use rustls::{
    internal::pemfile::{certs, rsa_private_keys},
    NoClientAuth, ServerConfig,
};
use std::{io, time::{Duration, Instant}};
use tokio_timer::Interval;

struct Sse {
    interval: Interval,
}

impl Stream for Sse {
    type Item = Bytes;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Bytes>, Error> {
        match self.interval.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(Some(Bytes::from(&b"data: ping\n"[..])))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Ok(Async::Ready(None)),
        }
    }
}

fn sse(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(Sse {
            interval: Interval::new(Instant::now(), Duration::from_millis(5000)),
        })
}

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
                if let Err(e) = push(&msg, &s) {
                    error!("Error: {}", e);
                }
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
    let mut uri: AMQPUri = format!("amqps://{}:{}@{}:{}/vhost",
        configuration.username, configuration.password,
        configuration.host, configuration.port).parse().unwrap();
    
    uri.cafile = Some(configuration.cafile);

    debug!("URI: {:?}", &uri);

    let fut = Client::connect_uri(uri, ConnectionProperties::default())
        .map_err(Error::from)
        .and_then(|client| {
            client.create_channel().map_err(Error::from)
        }).and_then(|channel| {
            let id = channel.id();
            info!("created channel with id: {}", id);

            let ch = channel.clone();
            channel.queue_declare("hello", QueueDeclareOptions::default(), FieldTable::default())
            .and_then(move |queue| {
                info!("channel {} declared queue {}", id, "hello");

                // basic_consume returns a future of a message
                // stream. Any time a message arrives for this consumer,
                // the for_each method would be called
                channel.basic_consume(&queue, "my_consumer", BasicConsumeOptions::default(), FieldTable::default())
            }).and_then(|stream| {
                info!("got consumer stream");
                let addr = SyncArbiter::start(2, || Summator);

                stream
                    .for_each(move |message| {
                        debug!("got message: {:?}", message.routing_key);
                        info!("raw message:\n{}", &*message.data.to_hex(16));

                        parse_header(&message.routing_key , message.properties.headers());

                        /*
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
                        */
                        ch.basic_ack(message.delivery_tag, false)
                    })
            }).map_err(Error::from)
        });

    // load ssl keys
    let config = load_ssl();

    HttpServer::new(|| App::new()
        .wrap(
            Cors::new() // <- Construct CORS middleware builder
                .allowed_origin("http://localhost:4200")
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600))
        .service(
            web::resource("/").to(index))
        .service(
            web::resource("/sse").to(sse))
        )
        .workers(2)
        .bind_rustls("0.0.0.0:8443", config)
        .expect("Can not bind to 0.0.0.0:8443")
        .keep_alive(15000)
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
