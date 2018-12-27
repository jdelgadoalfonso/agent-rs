use amq_protocol::uri::{AMQPQueryString, AMQPUserInfo};
use failure;
use futures::Future;
use lapin_futures::client::{Client, ConnectionOptions, HeartbeatHandle};
use std::{io, net::SocketAddr};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_tcp::TcpStream;
use trust_dns_resolver::ResolverFuture;

pub fn open_tcp_stream(
    host: &str,
    port: u16,
) -> Box<dyn Future<Item = TcpStream, Error = failure::Error> + 'static> {
    let host = String::from(host);
    Box::new(
        futures::future::result(ResolverFuture::from_system_conf())
            .flatten()
            .map_err(From::from)
            .and_then(move |resolver| resolver.lookup_ip(host.as_str()))
            .map_err(From::from)
            .and_then(move |response| {
                response.iter().next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::AddrNotAvailable, "Couldn't resolve hostname")
                })
            })
            .and_then(move |ipaddr| TcpStream::connect(&SocketAddr::new(ipaddr, port)))
            .map_err(From::from)
    )
}

pub fn connect_stream<
    T: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: FnOnce(lapin_futures::error::Error) + Send + 'static,
>(
    stream: T,
    credentials: AMQPUserInfo,
    vhost: String,
    query: &AMQPQueryString,
    heartbeat_error_handler: F,
) -> Box<
    dyn Future<Item = (Client<T>, Option<HeartbeatHandle>), Error = lapin_futures::error::Error>
        + Send
        + 'static,
> {
    let defaults = ConnectionOptions::default();

    Box::new(
        Client::connect(
            stream,
            ConnectionOptions {
                username: credentials.username,
                password: credentials.password,
                vhost: vhost,
                frame_max: query.frame_max.unwrap_or_else(|| defaults.frame_max),
                heartbeat: query.heartbeat.unwrap_or_else(|| defaults.heartbeat),
            },
        )
        .map(move |(client, mut heartbeat_future)| {
            let heartbeat_handle = heartbeat_future.handle();
            actix::spawn(heartbeat_future.map_err(heartbeat_error_handler));
            (client, heartbeat_handle)
        }),
    )
}
