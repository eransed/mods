use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0071 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0071 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0071(data: &str) -> Result<Mid0071, String> {
  let (header, data) = parse(data, 71)?;
  Ok(Mid0071 { header, data })
}
