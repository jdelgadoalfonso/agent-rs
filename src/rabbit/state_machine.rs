machine!(
    #[derive(Clone, Debug, PartialEq)]
    enum Rabbit {
        Offline,
        Connected,
        Configured,
        UpAndRunning,
    }
);

#[derive(Clone, Debug, PartialEq)]
pub struct Connect;

#[derive(Clone, Debug, PartialEq)]
pub struct TryReconnect;

#[derive(Clone, Debug, PartialEq)]
pub struct Configure;

#[derive(Clone, Debug, PartialEq)]
pub struct ConfigureDone;

#[derive(Clone, Debug, PartialEq)]
pub struct EventConnectionCreated;

#[derive(Clone, Debug, PartialEq)]
pub struct EventConnectionClosed;

#[derive(Clone, Debug, PartialEq)]
pub struct EventConsumerCreated;

#[derive(Clone, Debug, PartialEq)]
pub struct EventConsumerDeleted;

#[derive(Clone, Debug, PartialEq)]
pub struct EventChannelCreated;

#[derive(Clone, Debug, PartialEq)]
pub struct EventUserAuthSuccess;

#[derive(Clone, Debug, PartialEq)]
pub struct EventUserAuthFailure;

#[derive(Clone, Debug, PartialEq)]
pub struct EventAssignRoute;

#[derive(Clone, Debug, PartialEq)]
pub struct EventUnassignRoute;

#[derive(Clone, Debug, PartialEq)]
pub struct EventWithdraw;

#[derive(Clone, Debug, PartialEq)]
pub struct Disconnect;

#[derive(Clone, Debug, PartialEq)]
pub struct RemovePermanently;

transitions!(Rabbit,
  [
    (Offline, Connect) => Connected,
    (Connected, Configure) => Configured,
    (Configured, ConfigureDone) => UpAndRunning,

    (UpAndRunning, EventConnectionCreated) => UpAndRunning,
    (UpAndRunning, EventConnectionClosed) => UpAndRunning,
    (UpAndRunning, EventConsumerCreated) => UpAndRunning,
    (UpAndRunning, EventConsumerDeleted) => UpAndRunning,
    (UpAndRunning, EventChannelCreated) => UpAndRunning,
    (UpAndRunning, EventUserAuthSuccess) => UpAndRunning,
    (UpAndRunning, EventUserAuthFailure) => UpAndRunning,
    (UpAndRunning, EventAssignRoute) => UpAndRunning,
    (UpAndRunning, EventUnassignRoute) => UpAndRunning,
    (UpAndRunning, EventWithdraw) => UpAndRunning,
    (UpAndRunning, RemovePermanently) => UpAndRunning,

    (Connected, Disconnect) => Offline,
    (Configured, Disconnect) => Offline,
    (UpAndRunning, Disconnect) => Offline,
  ]
);

impl Offline {
    pub fn on_connect(self, _: Connect) -> Connected {
        Connected {}
    }
}

impl Connected {
    pub fn on_configure(self, _: Configure) -> Configured {
        Configured {}
    }

    pub fn on_disconnect(self, _: Disconnect) -> Offline {
        Offline {}
    }
}

impl Configured {
    pub fn on_configure_done(self, _: ConfigureDone) -> UpAndRunning {
        UpAndRunning {}
    }

    pub fn on_disconnect(self, _: Disconnect) -> Offline {
        Offline {}
    }
}

impl UpAndRunning {
    pub fn on_event_connection_created(self, _: EventConnectionCreated) -> UpAndRunning {
        self
    }

    pub fn on_event_connection_closed(self, _: EventConnectionClosed) -> UpAndRunning {
        self
    }

    pub fn on_event_consumer_created(self, _: EventConsumerCreated) -> UpAndRunning {
        self
    }

    pub fn on_event_consumer_deleted(self, _: EventConsumerDeleted) -> UpAndRunning {
        self
    }

    pub fn on_event_channel_created(self, _: EventChannelCreated) -> UpAndRunning {
        self
    }

    pub fn on_event_user_auth_success(self, _: EventUserAuthSuccess) -> UpAndRunning {
        self
    }

    pub fn on_event_user_auth_failure(self, _: EventUserAuthFailure) -> UpAndRunning {
        self
    }

    pub fn on_event_assign_route(self, _: EventAssignRoute) -> UpAndRunning {
        self
    }

    pub fn on_event_unassign_route(self, _: EventUnassignRoute) -> UpAndRunning {
        self
    }

    pub fn on_event_withdraw(self, _: EventWithdraw) -> UpAndRunning {
        self
    }

    pub fn on_remove_permanently(self, _: RemovePermanently) -> UpAndRunning {
        self
    }

    pub fn on_disconnect(self, _: Disconnect) -> Offline {
        Offline {}
    }
}

/*
methods!(Traffic,
  [
    Green => get count: u8,
    Green => set count: u8,
    [Green, Orange, Red,] => fn can_pass(&self) -> bool,
  ]
);

impl Green {
  pub fn can_pass(&self) -> bool {
    true
  }
}

impl Orange {
  pub fn can_pass(&self) -> bool {
    false
  }
}

impl Red {
  pub fn can_pass(&self) -> bool {
    false
  }
}
*/
