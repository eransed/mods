use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0073 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0073 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0073(data: &str) -> Result<Mid0073, String> {
  let (header, data) = parse(data, 73)?;
  Ok(Mid0073 { header, data })
}
