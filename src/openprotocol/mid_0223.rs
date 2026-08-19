use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0223 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0223 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0223(data: &str) -> Result<Mid0223, String> {
  let (header, data) = parse(data, 223)?;
  Ok(Mid0223 { header, data })
}
