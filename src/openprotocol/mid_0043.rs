use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0043 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0043 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0043(data: &str) -> Result<Mid0043, String> {
  let (header, data) = parse(data, 43)?;
  Ok(Mid0043 { header, data })
}
