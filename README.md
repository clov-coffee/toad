# kwap
A CoAP implementation that strives to power client- and server-side CoAP in any language & any environment.

## Project Goals
 - make coap accessible & approachable to those unfamiliar
 - headless CoAP core that can be used by frontends in any language (via JNI/C ABI/WASM)
 - support multi-role M2M communication (coap Endpoints must be able to act as both client & server)
 - make `async`, `alloc` & `std` _completely opt-in_ for clients & servers

## CoAP
CoAP is an application-level network protocol that copies the semantics of HTTP
to an environment conducive to **constrained** devices. (weak hardware, small battery capacity, etc.)

This means that you can write and run two-way RESTful communication
between devices very similarly to the networking semantics you are
most likely very familiar with.

### Similarities to HTTP
CoAP has the same verbs and many of the same semantics as HTTP;
- GET, POST, PUT, DELETE
- Headers (renamed to [Options](https://datatracker.ietf.org/doc/html/rfc7252#section-5.10))
- Data format independent (via the [Content-Format](https://datatracker.ietf.org/doc/html/rfc7252#section-12.3) Option)
- [Response status codes](https://datatracker.ietf.org/doc/html/rfc7252#section-5.9)

### Differences from HTTP
- CoAP customarily sits on top of UDP (however the standard is [in the process of being adapted](https://tools.ietf.org/id/draft-ietf-core-coap-tcp-tls-11.html) to also run on TCP, like HTTP)
- Because UDP is a "connectionless" protocol, it offers no guarantee of "conversation" between traditional client and server roles. All the UDP transport layer gives you is a method to listen for messages thrown at you, and to throw messages at someone. Owing to this, CoAP machines are expected to perform both client and server roles (or more accurately, _sender_ and _receiver_ roles)
- While _classes_ of status codes are the same (Success 2xx -> 2.xx, Client error 4xx -> 4.xx, Server error 5xx -> 5.xx), the semantics of the individual response codes differ.

## Work to be done
a `?` indicates that a feature is not blocking for a stable release, and may be implemented at a later date.

 - [ ] library structure
   - [x] standardize features
     - [x] `std` enables the standard library, and is enabled by default
     - [x] `--no-default-features` + `alloc` enables allocator without `std`
     - [x] `--no-default-features` disables allocator and std
   - [ ] `kwap_core` (_see [future library structure](#future-library-structure)_)
     - [ ] `Core`
     - [ ] `Config`
     - [ ] `Req`
     - [ ] `Resp`
     - [ ] `req::`
       - [ ] `Req`
       - [ ] `Method`
     - [ ] `resp::`
       - [ ] `Resp`
       - [ ] `code::`
   - [ ] `kwap::`
     - [ ] `ReqBuilder`
     - [ ] `RespBuilder`
     - [ ] `blocking::`
       - [ ] `Client`
       - [ ] `ClientBuilder`
       - [ ] `Server`
       - [ ] `ServerBuilder`
     - [ ] `async_std::`
       - [ ] `Client`
       - [ ] `ClientBuilder`
       - [ ] `Server`
       - [ ] `ServerBuilder`
   - [ ] `kwap_std::` (_very simple re-exporter for convenience on `std` platforms_)
     - [ ] `ReqBuilder` (_re-exports `kwap::ReqBuilder`_)
     - [ ] `RespBuilder` (_re-exports `kwap::RespBuilder`_)
     - [ ] `Client` (_re-exports `kwap::async_std::Client::<Std>`_)
     - [ ] `ClientBuilder` (_re-exports `kwap::async_std::ClientBuilder::<Std>`_)
     - [ ] `Server` (_re-exports `kwap::async_std::Server::<Std>`_)
     - [ ] `ServerBuilder` (_re-exports `kwap::async_std::ServerBuilder::<Std>`_)
     - [ ] `blocking::`
       - [ ] `Client` (_re-exports `kwap::blocking::Client::<Std>`_)
       - [ ] `ClientBuilder` (_re-exports `kwap::blocking::ClientBuilder::<Std>`_)
       - [ ] `Server` (_re-exports `kwap::blocking::Server::<Std>`_)
       - [ ] `ServerBuilder` (_re-exports `kwap::blocking::ServerBuilder::<Std>`_)
 - [x] parse messages
 - [x] ipv4
 - [ ] caching?
 - [ ] proxying?
 - [ ] ipv6?
 - [ ] multicast?
 - [ ] there exists a solution for dns resolution on embedded?
 - [ ] coaps? (coap over dtls)
 - [ ] observe
 - [ ] client flow
   - [x] send a ping message
   - [x] send confirmable requests
   - [x] send nonconfirmable requests
   - [x] retry send
   - [x] poll for matching piggybacked ack response
   - [x] poll for matching con response
   - [x] ack con response
   - [ ] send nons without expecting a response (fling nons into the void)
   - [ ] transmission variables (`ACK_TIMEOUT`, `ACK_RANDOM_FACTOR`, etc)
   - [ ] aggregate [`Block`](https://core-wg.github.io/new-block/draft-ietf-core-new-block.html)ed responses
   - [ ] support silently resending messages upon receiving a RESET to a CON or NON request
 - [ ] server flow
   - [ ] send piggybacked responses to requests
   - [ ] send separate ack & con resps
   - [ ] retry send resps
 - [ ] high-level `reqwest::Client` analogue
   - [x] blocking MVP that just sends requests
   - [ ] async MVP that just sends requests
   - [ ] support JSON
   - [ ] support CBOR
   - [ ] support configuring transmission variables
   - [ ] inline request building

### Future Library Structure
I plan on restructuring the modules soon(ish) to move the "core runtime" to its own crate to declutter
the module namespace and code footprint of `kwap`. This would leave `kwap` as a pleasant high-level crate for
rust users.
```
kwap_core
├── Config (config::Config)
├── config
│  ├── Alloc
│  ├── Config
│  └── Std
├── Core
├── Req (req::Req)
├── req
│  ├── Method
│  └── Req
├── Resp (resp::Resp)
└── resp
   ├── code
   │  ├── OK_CONTENT
   │  └── other codes...
   └── Resp

kwap
├── async_std
│  ├── Client
│  ├── ClientBuilder
│  ├── Server
│  └── ServerBuilder
├── blocking
│  ├── Client
│  ├── ClientBuilder
│  ├── Server
│  └── ServerBuilder
├── ReqBuilder
└── RespBuilder
```

## How it works (at the moment)
`kwap` contains the core CoAP runtime that drives client & server behavior.

It uses `kwap_common::Array` to stay decoupled from specific collection types (this makes `alloc` optional)

It uses `nb` to represent nonblocking async io (this will make `async` optional)

It represents the flow of messages through the system as a state machine, allowing for an open-ended system for customizing runtime behavior (this allows for writing idiomatic interfaces in other languages, e.g. invoking JS callbacks on request receipt)

#### Server flow
<details>
  <summary>Click to expand</summary>
  
```
RecvDgram
    |
 {parse}--------------------
    |                       |
    v                       v
 Recv{Ack,Empty,Request}  MsgParseErr
     |                      |
 {process}--------          |
     |            | <-------
     |      ----> |
     v     |      v
  MsgProcessErr  ToSend
                  |
               {send}
                  |<----------------------
                  |------                 |
                  |      |                |
                  v      v                |
                Done    SendErr --{retry}-
                                          |
                                          |
                                          v
                                     SendPoisoned
```
</details>

#### What a high-level rust interface may look like
<details>
<summary>Click to expand</summary>

```rust
fn main() {
  let udp: kwap::Sock = std::UdpSocket::bind(/* addr */).unwrap();
  let server = kwap::Server::new(sock).resource(Hello);

  server.start();
}

struct Hello;
impl kwap::Resource for Hello {
  const ID: kwap::ResourceId = kwap::ResourceId::from_str("Hello");

  fn should_handle(&self, req: kwap::Req) -> bool {
    req.path.get(0) == Some("hello")
  }

  fn handle(&self, server: &kwap::Server, req: kwap::Req) -> kwap::Result<kwap::Rep> {
    if !req.method.is_get() {
      return kwap::rep::error::method_not_allowed();
    }

    let name = req.get(1).unwrap_or("World");

    if name == "Jeff" {
      return kwap::rep::error::unauthorized("Jeff, I told you this isn't for you. Please leave.");
    }

    let payload = serde_json::json!({"msg": format!("Hello, {}", name)});

    kwap::rep::ok::content(payload)
  }
}
```
</details>
