use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0218 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0218 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0218(data: &str) -> Result<Mid0218, String> {
  let (header, data) = parse(data, 218)?;
  Ok(Mid0218 { header, data })
}
