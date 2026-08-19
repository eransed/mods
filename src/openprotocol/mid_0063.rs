use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0063 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0063 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0063(data: &str) -> Result<Mid0063, String> {
  let (header, data) = parse(data, 63)?;
  Ok(Mid0063 { header, data })
}
