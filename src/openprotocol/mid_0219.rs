use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0219 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0219 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0219(data: &str) -> Result<Mid0219, String> {
  let (header, data) = parse(data, 219)?;
  Ok(Mid0219 { header, data })
}
