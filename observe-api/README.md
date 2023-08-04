# Observe API

The *Observe API* is a set of *host* functions that each of our Host SDK supports.
This acts as the contract between the host and the guest layer. All data flows in one direction,
from the guest to the host. Most of these APIs are simply ways to pass observability data as strings
to the host layer.

* `dylibso_observe.metric(i32, i64, i32)`
* `dylibso_observe.log(i64, i32)`
* `dylibso_observe.span_enter(i64, i32)`
* `dylibso_observe.span_exit()`
* `dylibso_observe.span_tags(i64, i32)`

Ideally, you will not call this API layer directly but instead use language specific wrappers to call them. And for end users, eventually, open source observability clients will *export* data to this layer.



