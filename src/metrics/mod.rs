use prometheus::{IntGaugeVec, Opts};

pub struct Metrics {
    last_event: IntGaugeVec,
    last_event_timestamp: IntGaugeVec,
    last_confirmed_event: IntGaugeVec,
    last_confirmed_event_timestamp: IntGaugeVec,
    last_received_event: IntGaugeVec,
    last_received_event_timestamp: IntGaugeVec,
}

impl Metrics {
    pub fn last_event(&self) -> &IntGaugeVec {
        &self.last_event
    }
    pub fn last_confirmed_event(&self) -> &IntGaugeVec {
        &self.last_confirmed_event
    }
    pub fn last_event_timestamp(&self) -> &IntGaugeVec {
        &self.last_event_timestamp
    }
    pub fn last_confirmed_event_timestamp(&self) -> &IntGaugeVec {
        &self.last_confirmed_event_timestamp
    }
    pub fn last_received_event(&self) -> &IntGaugeVec {
        &self.last_received_event
    }
    pub fn last_received_event_timestamp(&self) -> &IntGaugeVec {
        &self.last_received_event_timestamp
    }
    pub fn new() -> Metrics {
        let opts = Opts::new("slony_confirmed_event", "The last event replicated");
        let last_confirmed_event =
            register_int_gauge_vec!(opts, &["slony_origin", "slony_receiver"]).unwrap();

        let opts = Opts::new(
            "slony_confirmed_event_timestamp",
            "The timestamp of the last event replicated and confirmed",
        );
        let last_confirmed_event_timestamp =
            register_int_gauge_vec!(opts, &["slony_origin", "slony_receiver"]).unwrap();

        let opts = Opts::new("slony_last_event", "The last event generated");
        let last_event = register_int_gauge_vec!(opts, &["slony_origin"]).unwrap();

        let opts = Opts::new(
            "slony_last_event_timestamp",
            "The timestamp of the last generated event",
        );
        let last_event_timestamp = register_int_gauge_vec!(opts, &["slony_origin"]).unwrap();

        let opts = Opts::new(
            "slony_received_event",
            "The last event received from a remote node",
        );
        let last_received_event =
            register_int_gauge_vec!(opts, &["slony_origin", "slony_receiver"]).unwrap();

        let opts = Opts::new(
            "slony_received_event_timestamp",
            "The timestamp of the last event replicated to this node from a remote node",
        );
        let last_received_event_timestamp =
            register_int_gauge_vec!(opts, &["slony_origin", "slony_receiver"]).unwrap();

        Metrics {
            last_confirmed_event: last_confirmed_event,
            last_event: last_event,
            last_event_timestamp: last_event_timestamp,
            last_confirmed_event_timestamp: last_confirmed_event_timestamp,
            last_received_event: last_received_event,
            last_received_event_timestamp: last_received_event_timestamp,
        }
    }
}
