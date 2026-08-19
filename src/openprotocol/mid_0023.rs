use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0023 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0023 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0023(data: &str) -> Result<Mid0023, String> {
  let (header, data) = parse(data, 23)?;
  Ok(Mid0023 { header, data })
}
