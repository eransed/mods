use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0252 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0252 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0252(data: &str) -> Result<Mid0252, String> {
  let (header, data) = parse(data, 252)?;
  Ok(Mid0252 { header, data })
}
