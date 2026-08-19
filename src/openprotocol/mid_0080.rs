use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0080 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0080 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0080(data: &str) -> Result<Mid0080, String> {
  let (header, data) = parse(data, 80)?;
  Ok(Mid0080 { header, data })
}
