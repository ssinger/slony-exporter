use postgres::{Client};
use postgres_openssl::MakeTlsConnector;
use openssl::ssl::{SslConnector,SslMethod};
use openssl::error::ErrorStack;
use std::collections::{HashSet};



pub struct SlonyStatus {
    //The node id of this node.
    node_id: i32,
    confirms: Vec<SlonyConfirm>,
    incoming: Vec<SlonyIncoming>,
    // The last event generated on this node
    last_event_id: i64,
    last_event_timestamp: i64,
    //Sets that this node is the origin for
    origin_sets: HashSet<i32>
}

pub struct SlonyConfirm {
    // #SlonyConfirms
    // This structure tracks confirmations from the origin node
    // to the indicated receiver.
    pub receiver: i32,
    pub last_confirmed_event: Option<i64>,
    pub last_confirmed_timestamp: Option<i64>,
}

pub struct SlonyIncoming {
    //# SlonyIncoming
    //This structure tracks incoming replication from the remote this
    //to this node.
    pub origin: i32,
    pub last_event_id: i64,
    pub last_event_timestamp: i64,
}

impl SlonyStatus {
    pub fn new(node_id: i32, last_event_id: i64, last_event_timestamp: i64) -> SlonyStatus {
        SlonyStatus {
            node_id,
            last_event_id,
            last_event_timestamp,
            confirms: vec![],
            incoming: vec![],
            origin_sets: HashSet::new()
        }
    }
    pub fn node_id(&self) -> i32 {
        self.node_id
    }
    pub fn last_event(&self) -> i64 {
        self.last_event_id
    }
    pub fn last_event_timestamp(&self) -> i64 {
        self.last_event_timestamp
    }
    pub fn confirms(&self) -> &Vec<SlonyConfirm> {
        &self.confirms
    }
    pub fn incoming(&self) -> &Vec<SlonyIncoming> {
        &self.incoming
    }
    pub fn origin_sets(&self) -> &HashSet<i32> {
        &self.origin_sets
    }
}

pub struct Error {
    message: String,
}

pub trait ErrorTrait {
    fn message(&self) -> &str;
}

impl ErrorTrait for Error {
    fn message(&self) -> &str {
        &self.message
    }
}

impl From<postgres::Error> for Error {
    fn from(error: postgres::Error) -> Self {
        Error {
            message: error.to_string(),
        }
    }
}

impl From<ErrorStack> for Error {
    fn from(error: ErrorStack) -> Self {
        Error {
            message: format!("SSL error: {} ",error)
        }
    }
}

fn connect(url: &str,tls_connector: MakeTlsConnector) -> Result<postgres::Client, postgres::Error> {    
    Client::connect(url, tls_connector)
}

pub fn fetch_slony_status() -> Result<SlonyStatus, Error> {
    let url_opt = std::env::var("POSTGRES_URL");
    let url = match url_opt {
        Ok(s) => s,
        Err(_e) => {
            return Err(Error {
                message: String::from("Must set POSTGRES_URL"),
            })
        }
    };

    query(&url)
}

fn query(url: &str) -> Result<SlonyStatus, Error> {
    let slony_schema_opt = std::env::var("SLONY_CLUSTER");
    let slony_schema = match slony_schema_opt {
        Ok(s) => s,
        Err(_e) => {
            return Err(Error {
                message: String::from("SLONY_CLUSTER must be set"),
            })
        }
    };

    let sslbuilder = SslConnector::builder(SslMethod::tls())?;
    let tls_connector = MakeTlsConnector::new(sslbuilder.build());
    let mut client = connect(url,tls_connector)?;

    // Fetch information about this node from sl_status.
    let slony_status = fetch_node_data(&mut client, &slony_schema)?;

    //Fetch confirmations
    let slony_status = fetch_node_confirmations(&mut client, &slony_schema, slony_status)?;

    let slony_status = fetch_node_incoming(&mut client, &slony_schema, slony_status)?;

    let slony_status = fetch_sets(&mut client, &slony_schema, slony_status)?;
    
    //Connection leak on error?
    let _r = client.close();
    Ok(slony_status)
}

fn fetch_node_data(client: &mut Client, slony_schema: &str) -> Result<SlonyStatus, Error> {
    //! # fetch_node_dat
    //! fetch information about the node that is being monitored.

    let query = format!(
        " select ev_origin, \
                          ev_seqno, \
                          extract(epoch from ev_timestamp)::int8 \
                          from _{}.sl_event \
                          where  \
                          ev_origin = _{}.getLocalNodeId($1)
                          order by ev_seqno desc limit 1",
        slony_schema, slony_schema
    );
    let event_rows = client.query(query.as_str(), &[&format!("_{}", slony_schema)])?;
    if let Some(row) = event_rows.get(0) {
        return Ok(SlonyStatus::new(row.get(0), row.get(1), row.get(2)));
    }

    Err(Error {
        message: String::from("No events found"),
    })
}

fn fetch_node_confirmations(
    client: &mut Client,
    slony_schema: &str,
    mut slony_status: SlonyStatus,
) -> Result<SlonyStatus, Error> {
    let query = format!("select con_received, \
                         con_seqno, \
                         extract(epoch from con_timestamp)::int8 \
                         from ( \
                              select con_received, \
                              con_seqno, \
                              con_timestamp, \
                              rank() over( partition by con_origin,con_received order by con_seqno desc)
                              from \
                              _{}.sl_confirm \
                              where \
                              con_origin=$1 \
                              ) as x \
                          where \
                          x.rank=1",slony_schema);
    let confirm_rows = client.query(query.as_str(), &[&slony_status.node_id()])?;
    let mut confirms: Vec<SlonyConfirm> = vec![];
    for row in confirm_rows {
        let confirm = SlonyConfirm {
            receiver: row.get(0),
            last_confirmed_event: row.get(1),
            last_confirmed_timestamp: row.get(2),
        };
        confirms.push(confirm);
    }
    slony_status.confirms = confirms;
    Ok(slony_status)
}

fn fetch_node_incoming(
    client: &mut Client,
    slony_schema: &str,
    mut slony_status: SlonyStatus,
) -> Result<SlonyStatus, Error> {
    let query = format!(
        " select ev_origin, \
                          ev_seqno, \
                          extract(epoch from ev_timestamp)::int8 \
                          from \
                          ( select ev_origin, \
                            ev_seqno, \
                            ev_timestamp, \
                            rank() over (partition by ev_origin order by ev_seqno desc) \
                            from _{}.sl_event \
                            where \
                            ev_origin<>$1 \
                            ) as x \
                            where x.rank=1 ",
        slony_schema
    );

    let incoming_rows = client.query(query.as_str(), &[&slony_status.node_id()])?;
    let mut incoming: Vec<SlonyIncoming> = vec![];
    for row in incoming_rows {
        let slony_incoming = SlonyIncoming {
            origin: row.get(0),
            last_event_id: row.get(1),
            last_event_timestamp: row.get(2),
        };
        incoming.push(slony_incoming);
    }
    slony_status.incoming = incoming;
    Ok(slony_status)
}



fn fetch_sets(
    client: &mut Client,
    slony_schema: &str,
    mut slony_status: SlonyStatus,
) -> Result<SlonyStatus, Error> {

    let query = format!("select set_id from _{}.sl_set where set_origin=$1",
                        slony_schema);
    let rows = client.query(query.as_str(),&[&slony_status.node_id()])?;
    let mut sets : HashSet<i32> = HashSet::new();
    for row in rows {
        sets.insert(row.get(0));
    }
    slony_status.origin_sets=sets;
    Ok(slony_status)
    
}
#[cfg(test)]
mod tests {
    use crate::slony::{SlonyConnection, SlonyStatus};
}
