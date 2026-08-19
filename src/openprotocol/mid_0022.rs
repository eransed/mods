use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0022 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0022 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0022(data: &str) -> Result<Mid0022, String> {
  let (header, data) = parse(data, 22)?;
  Ok(Mid0022 { header, data })
}
