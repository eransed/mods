use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid9999 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid9999 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_9999(data: &str) -> Result<Mid9999, String> {
  let (header, data) = parse(data, 9999)?;
  Ok(Mid9999 { header, data })
}
