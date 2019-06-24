use amq_protocol::types::{FieldTable, ShortString};
use std::str::FromStr;

#[derive(Debug, EnumString)]
enum RabbitEvent {
    #[strum(serialize = "connection.created")]
    ConnectionCreated,
    #[strum(serialize = "connection.closed")]
    ConnectionClosed,
    #[strum(serialize = "consumer.created")]
    ConsumerCreated,
    #[strum(serialize = "consumer.deleted")]
    ConsumerDeleted,
    #[strum(serialize = "channel.created")]
    ChannelCreated,
    #[strum(serialize = "user.authentication.success")]
    UserAuthSuccess,
    #[strum(serialize = "user.authentication.failure")]
    UserAuthFailure,
}

pub fn parse_header(routing_key: &ShortString, oft: &Option<FieldTable>) {
    if let Some(ft) = oft {
        ft.into_iter()
            .map(|item| {
                format!("{:?}", item)
            });
/*
        let name: Option<&AMQPValue> = ft.get("name");
        let timestamp: Option<&AMQPValue> = ft.get("timestamp_in_ms");
        let queue: Option<&AMQPValue> = ft.get("queue");
        let pid: Option<&AMQPValue> = ft.get("pid");
        let channel: Option<&AMQPValue> = ft.get("channel");
        let user: Option<&AMQPValue> = ft.get("user");
        let peer_host: Option<&AMQPValue> = ft.get("peer_host");
        let connection: Option<&AMQPValue> = ft.get("connection");
        let error: Option<&AMQPValue> = ft.get("error");
*/
        match RabbitEvent::from_str(routing_key.as_str()) {
            Ok(RabbitEvent::UserAuthSuccess) => {}
            Ok(RabbitEvent::UserAuthFailure) => {}
            Ok(RabbitEvent::ConnectionCreated) => {}
            Ok(RabbitEvent::ConnectionClosed) => {}
            Ok(RabbitEvent::ChannelCreated) => {}
            Ok(RabbitEvent::ConsumerCreated) => {}
            Ok(RabbitEvent::ConsumerDeleted) => {}
            Err(e) => { error!("{}, {}", e, routing_key); }
        };
    };
}
