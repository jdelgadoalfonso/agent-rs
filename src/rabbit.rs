extern crate abstract_ns;
extern crate futures;
extern crate lapin_futures as lapin;

use abstract_ns::HostResolve;

use actix::Arbiter;

use amq_protocol::uri::{AMQPQueryString, AMQPUserInfo};

use futures::Future;

use lapin::client::ConnectionOptions;

use ns_dns_tokio::DnsResolver;

use std::{io, str::FromStr};

use tokio_core::net::TcpStream;

use tokio_io::{AsyncRead, AsyncWrite};


pub fn open_tcp_stream(host: &str, port: u16) ->
Box<Future<Item = TcpStream, Error = io::Error> + 'static>
{
    let resolver = DnsResolver::system_config(Arbiter::handle())
    .map_err(|_err| io::Error::new(
        io::ErrorKind::Other, "Failed to initialize DnsResolver"
    ));
    let name = abstract_ns::name::Name::from_str(host)
    .map_err(|err| io::Error::new(
        io::ErrorKind::Other, err
    ));

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
            TcpStream::connect(&sockaddr, Arbiter::handle())
        })
    )
}

pub fn connect_stream
<T: AsyncRead + AsyncWrite + Send + Sync + 'static,
F: FnOnce(io::Error) + 'static>
(stream: T, credentials: AMQPUserInfo, vhost: String,
query: &AMQPQueryString, heartbeat_error_handler: F) ->
Box<Future<Item = lapin::client::Client<T>, Error = io::Error> + 'static>
{
    let defaults = ConnectionOptions::default();

    Box::new(lapin::client::Client::connect(stream, &ConnectionOptions {
        username:  credentials.username,
        password:  credentials.password,
        vhost:     vhost,
        frame_max: query.frame_max.unwrap_or_else(|| defaults.frame_max),
        heartbeat: query.heartbeat.unwrap_or_else(|| defaults.heartbeat),
    }).map(move |(client, heartbeat_future_fn)| {
        let heartbeat_client = client.clone();
        Arbiter::handle().spawn(heartbeat_future_fn(&heartbeat_client)
        .map_err(heartbeat_error_handler));
        client
    }))
}
