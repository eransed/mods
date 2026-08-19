use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0216 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0216 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0216(data: &str) -> Result<Mid0216, String> {
  let (header, data) = parse(data, 216)?;
  Ok(Mid0216 { header, data })
}
