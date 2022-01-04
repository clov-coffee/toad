use core::ops::{Deref, DerefMut};

use kwap_common::Array;
use kwap_msg::{Message, Opt, OptNumber, Payload, Type};
#[cfg(feature = "alloc")]
use std_alloc::{string::{FromUtf8Error, String},
                vec::Vec};

/// Response codes
pub mod code;

/// [`Resp`] that uses [`Vec`] as the backing collection type
///
/// ```
/// use kwap::resp::Resp;
/// # use kwap_msg::*;
/// # main();
///
/// fn main() {
///   start_server(|req| {
///     let mut resp = Resp::for_request(req);
///
///     resp.set_code(kwap::resp::code::CONTENT);
///     resp.set_option(12, [50]);
///
///     let payload = r#"""{
///       "foo": "bar",
///       "baz": "quux"
///     }"""#;
///     resp.set_payload(payload.bytes());
///
///     resp
///   });
/// }
///
/// fn start_server(f: impl FnOnce(kwap::req::Req) -> kwap::resp::Resp) {
///   // servery things
/// # f(kwap::req::Req::get("foo"));
/// }
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(any(docsrs, feature = "docs"), doc(cfg(feature = "alloc")))]
#[derive(Debug, Clone)]
pub struct Resp(VecRespCore);

impl Resp {
  /// Create a new response for a given request
  pub fn for_request(req: crate::req::Req) -> Self {
    Self(RespCore::for_request(req.0))
  }
}

impl Deref for Resp {
  type Target = VecRespCore;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Resp {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

type VecRespCore = RespCore<Vec<u8>, Vec<u8>, Vec<Opt<Vec<u8>>>, Vec<(OptNumber, Opt<Vec<u8>>)>>;

/// TODO: ser/de support
#[derive(Clone, Debug)]
pub struct RespCore<Bytes: Array<Item = u8>,
 OptBytes: Array<Item = u8> + 'static,
 Opts: Array<Item = Opt<OptBytes>>,
 OptNumbers: Array<Item = (OptNumber, Opt<OptBytes>)>>
{
  msg: Message<Bytes, OptBytes, Opts>,
  opts: Option<OptNumbers>,
}

impl<Bytes: Array<Item = u8>,
      OptBytes: Array<Item = u8> + 'static,
      Opts: Array<Item = Opt<OptBytes>>,
      OptNumbers: Array<Item = (OptNumber, Opt<OptBytes>)>> RespCore<Bytes, OptBytes, Opts, OptNumbers>
{
  /// Create a new response for a given request
  ///
  /// TODO: replace msg with Request type
  pub fn for_request(req: crate::req::ReqCore<Bytes, OptBytes, Opts, OptNumbers>) -> Self {
    let req = Message::from(req);

    let msg = Message { ty: match req.ty {
                          | Type::Con => Type::Ack,
                          | _ => req.ty,
                        },
                        id: if req.ty == Type::Con {
                          req.id
                        } else {
                          crate::generate_id()
                        },
                        opts: Opts::default(),
                        code: code::CONTENT,
                        ver: Default::default(),
                        payload: Payload(Default::default()),
                        token: req.token,
                        };

    Self { msg,
           opts: None }
  }

  /// Get the payload's raw bytes
  pub fn payload(&self) -> impl Iterator<Item = &u8> {
    self.msg.payload.0.iter()
  }

  /// Get the payload and attempt to interpret it as an ASCII string
  #[cfg(feature = "alloc")]
  pub fn payload_string(&self) -> Result<String, FromUtf8Error> {
    String::from_utf8(self.payload().copied().collect())
  }

  /// Change the response code
  pub fn set_code(&mut self, code: kwap_msg::Code) {
    self.msg.code = code;
  }

  /// Add a custom option to the response
  ///
  /// If there was no room in the collection, returns the arguments back as `Some(number, value)`.
  /// Otherwise, returns `None`.
  pub fn set_option<V: IntoIterator<Item = u8>>(&mut self, number: u32, value: V) -> Option<(u32, V)> {
    if self.opts.is_none() {self.opts = Some(Default::default());}
    crate::add_option(self.opts.as_mut().unwrap(), number, value)
  }

  /// Add a payload to this response
  pub fn set_payload<P: IntoIterator<Item = u8>>(&mut self, payload: P) {
    self.msg.payload = Payload(payload.into_iter().collect());
  }

  /// Drains the internal associated list of opt number <> opt and converts the numbers into deltas to prepare for message transmission
  fn normalize_opts(&mut self) {
    if let Some(opts) = Option::take(&mut self.opts) {
      self.msg.opts = crate::normalize_opts(opts);
    }
  }
}
