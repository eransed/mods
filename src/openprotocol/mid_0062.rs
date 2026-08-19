use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0062 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0062 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0062(data: &str) -> Result<Mid0062, String> {
  let (header, data) = parse(data, 62)?;
  Ok(Mid0062 { header, data })
}
