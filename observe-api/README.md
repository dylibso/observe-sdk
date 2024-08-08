# Observe API

The *Observe API* is a set of *host* functions that are supported by each of our Host SDKs.
This acts as the contract between the host and the guest layer. All data flows in one direction,
from the guest to the host. Most of these APIs are simply ways to pass observability data as strings
to the host layer.

* `dylibso:observe/api.metric(i32, i32, i32)`
* `dylibso:observe/api.log(i32, i32, i32)`
* `dylibso:observe/api.span-enter(i32, i32)`
* `dylibso:observe/api.span-exit()`
* `dylibso:observe/api.span-tags(i32, i32)`

Ideally, you will not call this API layer directly but instead use language specific bindings to call them. And for end users, eventually, open source observability clients will *export* data to this layer.

## Language Bindings

We currently provide these language bindings to this API:

### [rust](rust/)

* [example](test/rust/src/main.rs)


### [c](c/) and [c++](cxx/)

Both the C and C++ bindings are implemented as single header libraries. To use the C bindings,
in __ONE__ source file:

```c
#define OBSERVE_API_IMPLEMENTATION
#include "observe_api.h"
```

In other source files, just `#include "observe_api.h"`

* [example](test/c/main.c)

To use the C++ bindings, instead, in __ONE__ source file:

```c++
#define OBSERVE_API_CPP_IMPLEMENTATION
#include "observe_api.hpp"
```

In other source files, just `#include "observe_api.hpp"`

* [functional example](test/cxx/main.cpp)
* [OO example](test/cxx/main2.cpp)

In C++, both bindings may be used at the same time without conflict.

### Other

More languages will come soon as well as tools built on top of these bindings. If you are planning on building your own tooling we suggest using or contributing one of these language specific bindings.

