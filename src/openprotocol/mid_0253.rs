use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0253 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0253 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0253(data: &str) -> Result<Mid0253, String> {
  let (header, data) = parse(data, 253)?;
  Ok(Mid0253 { header, data })
}
