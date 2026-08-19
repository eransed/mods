use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0215 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0215 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0215(data: &str) -> Result<Mid0215, String> {
  let (header, data) = parse(data, 215)?;
  Ok(Mid0215 { header, data })
}
