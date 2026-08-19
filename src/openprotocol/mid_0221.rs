use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0221 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0221 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0221(data: &str) -> Result<Mid0221, String> {
  let (header, data) = parse(data, 221)?;
  Ok(Mid0221 { header, data })
}
