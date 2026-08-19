use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0217 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0217 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0217(data: &str) -> Result<Mid0217, String> {
  let (header, data) = parse(data, 217)?;
  Ok(Mid0217 { header, data })
}
