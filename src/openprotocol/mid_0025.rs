use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0025 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0025 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0025(data: &str) -> Result<Mid0025, String> {
  let (header, data) = parse(data, 25)?;
  Ok(Mid0025 { header, data })
}
