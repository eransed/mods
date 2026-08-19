use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0065 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0065 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0065(data: &str) -> Result<Mid0065, String> {
  let (header, data) = parse(data, 65)?;
  Ok(Mid0065 { header, data })
}
