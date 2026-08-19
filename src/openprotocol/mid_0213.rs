use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0213 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0213 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0213(data: &str) -> Result<Mid0213, String> {
  let (header, data) = parse(data, 213)?;
  Ok(Mid0213 { header, data })
}
