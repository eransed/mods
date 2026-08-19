use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0251 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0251 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0251(data: &str) -> Result<Mid0251, String> {
  let (header, data) = parse(data, 251)?;
  Ok(Mid0251 { header, data })
}
