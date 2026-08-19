use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0210 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0210 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0210(data: &str) -> Result<Mid0210, String> {
  let (header, data) = parse(data, 210)?;
  Ok(Mid0210 { header, data })
}
