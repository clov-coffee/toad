use super::*;
use crate::is_full::IsFull;

impl<'a, const PAYLOAD_CAP: usize, const N_OPTS: usize, const OPT_CAP: usize> TryFromBytes<&'a u8>
  for Message<PAYLOAD_CAP, N_OPTS, OPT_CAP>
{
  type Error = MessageParseError;
  fn try_from_bytes<I: IntoIterator<Item = &'a u8>>(bytes: I) -> Result<Self, Self::Error> {
    Self::try_from_bytes(bytes.into_iter().copied())
  }
}

impl<const PAYLOAD_CAP: usize, const N_OPTS: usize, const OPT_CAP: usize> TryFromBytes<u8>
  for Message<PAYLOAD_CAP, N_OPTS, OPT_CAP>
{
  type Error = MessageParseError;

  fn try_from_bytes<I: IntoIterator<Item = u8>>(bytes: I) -> Result<Self, Self::Error> {
    let mut bytes = bytes.into_iter();

    let Byte1 { tkl, ty, ver } = Self::Error::try_next(&mut bytes)?.into();

    if tkl > 8 {
      return Err(Self::Error::InvalidTokenLength(tkl));
    }

    let code: Code = Self::Error::try_next(&mut bytes)?.into();
    let id: Id = Id::try_consume_bytes(&mut bytes)?;
    let token = Token::try_consume_bytes(&mut bytes.by_ref().take(tkl as usize))?;
    let opts = ArrayVec::<[Opt<OPT_CAP>; N_OPTS]>::try_consume_bytes(&mut bytes).map_err(Self::Error::OptParseError)?;
    let mut payload_bytes = ArrayVec::new();
    for byte in bytes {
      if let Some(_) = payload_bytes.try_push(byte) {
        return Err(Self::Error::PayloadTooLong(PAYLOAD_CAP));
      }
    }

    let payload = Payload(payload_bytes);

    Ok(Message { id,
                 ty,
                 ver,
                 code,
                 token,
                 opts,
                 payload })
  }
}







#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_msg() {
    let (expect, msg) = super::super::test_msg();
    assert_eq!(Message::<13, 1, 16>::try_from_bytes(&msg).unwrap(), expect)
  }

  #[test]
  fn parse_byte1() {
    let byte = 0b_01_10_0011u8;
    let byte = Byte1::from(byte);
    assert_eq!(byte,
               Byte1 { ver: Version(1),
                       ty: Type(2),
                       tkl: 3 })
  }

  #[test]
  fn parse_id() {
    let id_bytes = 34u16.to_be_bytes();
    let id = Id::try_consume_bytes(&mut id_bytes.iter().copied()).unwrap();
    assert_eq!(id, Id(34));
  }

  #[test]
  fn parse_code() {
    let byte = 0b_01_000101u8;
    let code = Code::from(byte);
    assert_eq!(code, Code { class: 2, detail: 5 })
  }

  #[test]
  fn parse_token() {
    let valid_a: [u8; 1] = [0b_00000001u8];
    let valid_a = Token::try_consume_bytes(&mut valid_a.iter().copied()).unwrap();
    assert_eq!(valid_a, Token(tinyvec::array_vec!([u8; 8] => 1)));
  }
}
