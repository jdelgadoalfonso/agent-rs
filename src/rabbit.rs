use crate::config::{client_config_with_root_ca, Configuration};
use amq_protocol::uri::{AMQPQueryString, AMQPUserInfo};
use failure::Error;
use futures::Future;
use lapin_futures::client::{Client, ConnectionOptions, HeartbeatHandle};
use std::{io, net::SocketAddr};
use tls_api::TlsConnectorBuilder as TraitTlsConnectorBuilder;
use tls_api_rustls::{TlsConnector, TlsConnectorBuilder};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_tcp::TcpStream;
use tokio_tls_api::TlsStream;
use trust_dns_resolver::AsyncResolver;

pub fn connect_to_rabbit_tls<F: FnOnce(lapin_futures::error::Error) + Send + 'static>(
    configuration: &Configuration,
    heartbeat_error_handler: F,
) -> Box<
    dyn Future<Item = (Client<Box<TlsStream<TcpStream>>>, Option<HeartbeatHandle>), Error = Error>,
> {
    let host = configuration.host.clone();
    let port = configuration.port;
    let cafile = configuration.cafile.clone();
    let vhost = configuration.vhost.clone();
    let userinfo = AMQPUserInfo {
        username: configuration.username.clone(),
        password: configuration.password.clone(),
    };
    let query = AMQPQueryString {
        frame_max: None,
        heartbeat: Some(20),
    };
    Box::new(
        open_tcp_stream(&host, port, &cafile)
            .and_then(move |(stream, connector)| {
                tokio_tls_api::connect_async(&connector, &host, stream)
                    .map_err(Error::from)
                    .map(Box::new)
            })
            .and_then(move |stream| {
                connect_stream(stream, userinfo, vhost, &query, heartbeat_error_handler)
            }),
    )
}

fn open_tcp_stream(
    host: &str,
    port: u16,
    cafile: &String,
) -> Box<dyn Future<Item = (TcpStream, TlsConnector), Error = Error> + 'static> {
    let host = String::from(host);
    Box::new(
        futures::future::result(AsyncResolver::from_system_conf())
            .and_then(move |(resolver, bg)| {
                actix::spawn(bg);
                resolver.lookup_ip(host.as_str())
            })
            .map_err(From::from)
            .and_then(move |response| {
                response.iter().next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::AddrNotAvailable, "Couldn't resolve hostname")
                })
            })
            .and_then(move |ipaddr| TcpStream::connect(&SocketAddr::new(ipaddr, port)))
            .map_err(From::from)
            .join({
                futures::future::result(
                    TlsConnectorBuilder(client_config_with_root_ca(cafile)).build(),
                )
            })
            .map_err(From::from),
    )
}

fn connect_stream<
    T: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: FnOnce(lapin_futures::error::Error) + Send + 'static,
>(
    stream: T,
    credentials: AMQPUserInfo,
    vhost: String,
    query: &AMQPQueryString,
    heartbeat_error_handler: F,
) -> Box<dyn Future<Item = (Client<T>, Option<HeartbeatHandle>), Error = Error> + Send + 'static> {
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
        })
        .map_err(Error::from),
    )
}
