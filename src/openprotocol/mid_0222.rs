use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0222 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0222 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0222(data: &str) -> Result<Mid0222, String> {
  let (header, data) = parse(data, 222)?;
  Ok(Mid0222 { header, data })
}
