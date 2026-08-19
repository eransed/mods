use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0024 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0024 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0024(data: &str) -> Result<Mid0024, String> {
  let (header, data) = parse(data, 24)?;
  Ok(Mid0024 { header, data })
}
