use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0052 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0052 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0052(data: &str) -> Result<Mid0052, String> {
  let (header, data) = parse(data, 52)?;
  Ok(Mid0052 { header, data })
}
