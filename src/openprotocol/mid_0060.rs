use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0060 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0060 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0060(data: &str) -> Result<Mid0060, String> {
  let (header, data) = parse(data, 60)?;
  Ok(Mid0060 { header, data })
}
