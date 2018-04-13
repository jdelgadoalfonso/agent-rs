#![feature(panic_info_message)]

extern crate abstract_ns;
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


use abstract_ns::HostResolve;

use amq_protocol::uri::{AMQPQueryString, AMQPUserInfo};

use config::{client_config_with_root_ca, config};

use futures::{Stream, Future};

use lapin::types::FieldTable;
use lapin::client::ConnectionOptions;
use lapin::channel::{
    BasicConsumeOptions, ConfirmSelectOptions, QueueDeclareOptions
};

use nom::{HexDisplay, IResult};

use ns_dns_tokio::DnsResolver;

use parser::parser;

use std::io;
use std::str::FromStr;

use tls_api::TlsConnectorBuilder;

use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpStream;

use tokio_io::{AsyncRead, AsyncWrite};


fn open_tcp_stream
(handle: &Handle, host: &str, port: u16) ->
Box<Future<Item = TcpStream, Error = io::Error> + 'static> {

    let resolver = DnsResolver::system_config(handle)
    .map_err(|_err| io::Error::new(
        io::ErrorKind::Other, "Failed to initialize DnsResolver"
    ));
    let name = abstract_ns::name::Name::from_str(host)
    .map_err(|err| io::Error::new(
        io::ErrorKind::Other, err
    ));
    let handle2 = handle.clone();

    Box::new(
        futures::future::result(resolver)
        .join(futures::future::result(name))
        .and_then(move |(resolver, name)| {
            resolver.resolve_host(&name).map_err(|err|
                io::Error::new(io::ErrorKind::Other, err)
            )
        }).map(move |ip_list| {
            ip_list.with_port(port)
        }).and_then(move |address| {
            address.pick_one().ok_or_else(|| io::Error::new(
                io::ErrorKind::AddrNotAvailable, "Couldn't resolve hostname"
            ))
        }).and_then(move |sockaddr| {
            TcpStream::connect(&sockaddr, &handle2)
        })
    )
}

fn connect_stream
<T: AsyncRead + AsyncWrite + Send + Sync + 'static,
 F: FnOnce(io::Error) + 'static>
(stream: T, handle: Handle, credentials: AMQPUserInfo,
 vhost: String, query: &AMQPQueryString, heartbeat_error_handler: F) ->
Box<Future<Item = lapin::client::Client<T>, Error = io::Error> + 'static> {

    let defaults = ConnectionOptions::default();
    Box::new(lapin::client::Client::connect(stream, &ConnectionOptions {
        username:  credentials.username,
        password:  credentials.password,
        vhost:     vhost,
        frame_max: query.frame_max.unwrap_or_else(|| defaults.frame_max),
        heartbeat: query.heartbeat.unwrap_or_else(|| defaults.heartbeat),
    }).map(move |(client, heartbeat_future_fn)| {
        let heartbeat_client = client.clone();
        handle.spawn(heartbeat_future_fn(&heartbeat_client)
        .map_err(heartbeat_error_handler));
        client
    }))
}

fn main() {
    // create the reactor
    let mut core = Core::new().unwrap();
    let handle = core.handle();
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

    let fut = open_tcp_stream(&handle, &configuration.host, configuration.port)
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
        connect_stream(
            stream, handle, userinfo, vhost, &query, |_| ()
        )
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

    match core.run(fut) {
        Ok(_) => info!("Ok"),
        Err(e) => error!("{:?}", e),
    }
}
