use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0051 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0051 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0051(data: &str) -> Result<Mid0051, String> {
  let (header, data) = parse(data, 51)?;
  Ok(Mid0051 { header, data })
}
