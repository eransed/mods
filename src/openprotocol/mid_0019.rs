use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0019 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0019 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0019(data: &str) -> Result<Mid0019, String> {
  let (header, data) = parse(data, 19)?;
  Ok(Mid0019 { header, data })
}
