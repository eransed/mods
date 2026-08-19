use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0066 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0066 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0066(data: &str) -> Result<Mid0066, String> {
  let (header, data) = parse(data, 66)?;
  Ok(Mid0066 { header, data })
}
