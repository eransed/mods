use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0212 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0212 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0212(data: &str) -> Result<Mid0212, String> {
  let (header, data) = parse(data, 212)?;
  Ok(Mid0212 { header, data })
}
