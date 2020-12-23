mod metrics;
mod slony;
use crate::slony::ErrorTrait;
use metrics::Metrics;
use prometheus::{Encoder, TextEncoder};
use slony::{SlonyStatus};

use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
#[macro_use]
extern crate prometheus;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref METRICS: Metrics = Metrics::new();
}

async fn metric(_r: Request<Body>) -> Result<Response<Body>, Infallible> {
    let r = slony::fetch_slony_status();
    if let Ok(ref slony) = r {
        metric_for_connection(slony, &METRICS);
    }
    match r {
        Ok(s) => Ok(Response::new(Body::from(render(s)))),
        Err(x) => Ok(Response::builder()
            .status(500)
            .body(Body::from(String::from(x.message())))
            .unwrap()),
    }
}

#[tokio::main]
pub async fn main() {
    METRICS.last_event();
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(|request| metric(request)))
    });
    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    let _r  = server.await;
}

/**
 * Creates metrics for each slony node pair.
 *
 */
fn render(_slony_status: SlonyStatus) -> String {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_family = prometheus::gather();
    encoder.encode(&metric_family, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

fn metric_for_connection(slony_connection: &SlonyStatus, metrics: &Metrics) {
    metrics
        .last_event()
        .with_label_values(&[&format!("{}", slony_connection.node_id())])
        .set(slony_connection.last_event());
    metrics
        .last_event_timestamp()
        .with_label_values(&[&format!("{}", slony_connection.node_id())])
        .set(slony_connection.last_event_timestamp());

    for confirm in slony_connection.confirms() {
        if let Some(v) = confirm.last_confirmed_event {
            metrics
                .last_confirmed_event()
                .with_label_values(&[
                    &format!("{}", slony_connection.node_id()),
                    &format!("{}", confirm.receiver),
                ])
                .set(v);
        }
        if let Some(v) = confirm.last_confirmed_timestamp {
            metrics
                .last_confirmed_event_timestamp()
                .with_label_values(&[
                    &format!("{}", slony_connection.node_id()),
                    &format!("{}", confirm.receiver),
                ])
                .set(v);
        }
    }

    for incoming in slony_connection.incoming() {
        metrics
            .last_received_event()
            .with_label_values(&[
                &format!("{}", slony_connection.node_id()),
                &format!("{}", incoming.origin),
            ])
            .set(incoming.last_event_id);
        metrics
            .last_received_event_timestamp()
            .with_label_values(&[
                &format!("{}", slony_connection.node_id()),
                &format!("{}", incoming.origin),
            ])
            .set(incoming.last_event_timestamp);
    }
}
