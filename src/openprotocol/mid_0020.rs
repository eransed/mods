use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0020 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0020 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0020(data: &str) -> Result<Mid0020, String> {
  let (header, data) = parse(data, 20)?;
  Ok(Mid0020 { header, data })
}
