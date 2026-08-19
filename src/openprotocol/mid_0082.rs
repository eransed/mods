use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0082 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0082 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0082(data: &str) -> Result<Mid0082, String> {
  let (header, data) = parse(data, 82)?;
  Ok(Mid0082 { header, data })
}
