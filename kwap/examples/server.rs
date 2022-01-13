use std::net::UdpSocket;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use kwap::config::{self, Alloc};
use kwap::req::{Method, Req};
use kwap::resp::{code, Resp};
use kwap_msg::{TryFromBytes, TryIntoBytes};

static mut SHUTDOWN: Option<(Sender<()>, Receiver<()>)> = None;

pub fn shutdown() {
  unsafe {
    SHUTDOWN.as_ref().unwrap().0.send(()).unwrap();
  }
}

fn should_shutdown() -> bool {
  unsafe { SHUTDOWN.as_ref().unwrap().1.try_recv().is_ok() }
}

pub fn spawn() -> JoinHandle<()> {
  std::thread::spawn(|| {
    let p = std::panic::catch_unwind(|| {
      server_main();
    });

    if p.is_err() {
      eprintln!("server panicked! {:?}", p);
    }
  })
}

fn server_main() {
  unsafe {
    SHUTDOWN = Some(channel());
  }

  let sock = UdpSocket::bind("0.0.0.0:5683").unwrap();
  sock.set_nonblocking(true).unwrap();
  let mut buf = [0u8; 1152];

  println!("server: up");

  loop {
    if should_shutdown() {
      println!("server: shutting down...");
      break;
    }

    match sock.recv_from(&mut buf) {
      | Ok((n, addr)) => {
        println!("server: got {} bytes", n);
        let msg = config::Message::<Alloc>::try_from_bytes(buf.iter().copied().take(n)).unwrap();
        let req = Req::<Alloc>::from(msg);
        let path = req.get_option(11)
                      .as_ref()
                      .map(|o| &o.value.0)
                      .map(|b| std::str::from_utf8(&b).unwrap());

        println!("server: got {} {:?}", req.method(), path);

        if req.method() == Method::GET && path == Some("hello") {
          let mut resp = Resp::<Alloc>::for_request(req);
          resp.set_payload("hello, world!".bytes());
          resp.set_code(code::CONTENT);

          sock.send_to(&resp.try_into_bytes::<Vec<u8>>().unwrap(), addr).unwrap();
        } else if req.method() == Method::EMPTY && req.opts().next().is_none() && req.payload().is_empty() {
          let mut resp = Resp::<Alloc>::for_request(req);
          resp.set_code(kwap_msg::Code::new(0, 0));

          let mut msg = config::Message::<Alloc>::from(resp);
          msg.ty = kwap_msg::Type::Reset;

          sock.send_to(&msg.try_into_bytes::<Vec<u8>>().unwrap(), addr).unwrap();
        } else {
          let mut resp = Resp::for_request(req);
          resp.set_code(code::NOT_FOUND);

          sock.send_to(&resp.try_into_bytes::<Vec<u8>>().unwrap(), addr).unwrap();
        }
      },
      | Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
      | Err(e) => panic!("{:?}", e),
    }
  }
}

fn main() {
  spawn().join().unwrap();
}
