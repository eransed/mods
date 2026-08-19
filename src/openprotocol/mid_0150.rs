use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0150 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0150 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0150(data: &str) -> Result<Mid0150, String> {
  let (header, data) = parse(data, 150)?;
  Ok(Mid0150 { header, data })
}
