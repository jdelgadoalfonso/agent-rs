
pub mod actor;
pub mod header_parser;
pub mod state_machine;

/*
type BoxxedConn = Box<TlsStream<TcpStream>>;
type ConfigRabbitConnection = (Channel, Consumer);
pub fn config_rabbit_connection<F: FnOnce(LapinError) + Send + 'static>(
    configuration: &Configuration,
    heartbeat_error_handler: F,
) -> Box<dyn Future<Item = ConfigRabbitConnection, Error = Error>> {
    Box::new(
        connect_to_rabbit_tls(&configuration, |e| error!("{:?}", e))
            .and_then(|client| {
                client
                    .create_confirm_channel(ConfirmSelectOptions::default())
                    .map_err(Error::from)
            })
            .and_then(|channel| {
                info!("created channel with id: {}", channel.id);
                channel
                    .queue_declare("hello", QueueDeclareOptions::default(), FieldTable::default())
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
                        FieldTable::default(),
                    )
                    .map_err(Error::from)
                    .map(move |stream| (channel, stream))
            }),
    )
}

fn connect_to_rabbit_tls<F: FnOnce(LapinError) + Send + 'static>(
    configuration: &Configuration,
    heartbeat_error_handler: F,
) -> Box<dyn Future<Item = Client, Error = Error>> {
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
        channel_max: None,
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

type StreamItem = (TcpStream, TlsConnector);

fn open_tcp_stream(
    host: &str,
    port: u16,
    cafile: &String,
) -> Box<dyn Future<Item = StreamItem, Error = Error> + 'static> {
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
                    TlsConnectorBuilder {
                        config: client_config_with_root_ca(cafile),
                        verify_hostname: false,
                    }
                    .build(),
                )
            })
            .map_err(From::from),
    )
}

fn connect_stream<F>(
    stream: Box<TlsStream<TcpStream>>,
    credentials: AMQPUserInfo,
    vhost: String,
    query: &AMQPQueryString,
    heartbeat_error_handler: F,
) -> Box<dyn Future<Item = Client, Error = Error> + Send + 'static>
where
    F: FnOnce(LapinError) + Send + 'static,
{
    let defaults = ConnectionProperties::default();
    let client_properties: BTreeMap<String, AMQPValue> = {
        username: credentials.username,
        password: credentials.password,
        vhost: vhost,
        frame_max: query.frame_max.unwrap_or_else(|| defaults.frame_max),
        heartbeat: query.heartbeat.unwrap_or_else(|| defaults.heartbeat),
        properties: ConnectionProperties::default(),
    };
    let p = AmqpTcpStream::from(stream);
    
    let client = Client::connect(
        stream,
        ConnectionProperties {
            client_properties: FieldTable::from(client_properties),
            ...default
        })
        .map_err(Error::from);

    Box::new(client)
}
*/