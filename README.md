# Slony Exporter

Promtheus exporter for a [Slony](https://www.slony.info) node

## Usage

~~~
export SLONY_CLUSTER=clustername
export POSTGRES_URL=postgresql://postgres@localhost:5432/test1
export PORT=9090
slony-exporter
~~~

The slony-exporter will listen for HTTP requests on port 9090 and respond with prometheus metrics as follows

|Metric name|labels|Description|
|-----------|------|-----------|
|slony_last_event|slony_origin|The last event generated on this node|
|slony_last_event_timestamp|slony_origin|The timestamp of the last event generated on the node|
|slony_confirmed_event|slony_origin,slony_receiver|The last event from slony origin(this node) that has been confirmed on the receiver and the confirmation has propogated back to this node(the origin)|
|slony_confirmed_event_timestamp|slony_origin,slony_receiver|The timestamp of when the last confirmed event was confirmed on the receiver|
|slony_received_event|slony_origin,slony_receiver|The last event from the remote origin(slony_origin) received by this node|
|slony_received_event_timestamp|The time the last event from slony_origin that has been received by this node hawas generated at. This is the time an event(which has propogated to this node, slony_receiver) was generated on a remote node(slony_origin)|
|slony_origin_sets|The replication sets that have slony_origin as the set origin. Each has a label of slony_set of the set id with a value of 1 if slony_origin is the origin of that set|

## Limitations

* HTTPS is not supported
* HTTP authentication is not supported
* Specifying SSL options for the SSL connection to postgresql including a key, client cert or CA is not
supported.