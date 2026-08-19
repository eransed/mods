use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0081 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0081 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0081(data: &str) -> Result<Mid0081, String> {
  let (header, data) = parse(data, 81)?;
  Ok(Mid0081 { header, data })
}
