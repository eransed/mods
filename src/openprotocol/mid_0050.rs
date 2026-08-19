use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0050 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0050 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0050(data: &str) -> Result<Mid0050, String> {
  let (header, data) = parse(data, 50)?;
  Ok(Mid0050 { header, data })
}
