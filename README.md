# kwap
An extensible multi-platform rusty implementation of CoAP

## What does this solve?
 - support multi-role M2M communication (endpoints must act as both client & server)
 - make coap _accessible_
 - make async _optional_
 - make alloc & std _optional_
 - provide a system that can be easily _extended_

## Under the hood
 - asynchronous event-driven architecture:
   - `fn on(srv: &Server, e: Event, f: fn(&Server, Event) -> Event) -> ()`
   - `Nop`
   - `RecvDgram(Vec<u8>)`
   - `MsgParseErr(kwap::packet::ParseError)`
   - `RecvAck(kwap::msg::Ack)`
   - `RecvEmpty(kwap::msg::Empty)`
   - `RecvRequest(kwap::req::Req)`
   - `GetRespErr(kwap::msg::Msg, kwap::Error)`
   - `ResourceChanged(kwap::ResourceId)`
   - `ToSend(kwap::msg::Msg)`
   - `SendErr(kwap::resp::Resp, kwap::Error)`
   - `SendPoisoned(kwap::resp::Resp)`

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
