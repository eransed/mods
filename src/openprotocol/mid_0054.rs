use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0054 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0054 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0054(data: &str) -> Result<Mid0054, String> {
  let (header, data) = parse(data, 54)?;
  Ok(Mid0054 { header, data })
}
