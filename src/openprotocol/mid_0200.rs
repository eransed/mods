use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0200 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0200 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0200(data: &str) -> Result<Mid0200, String> {
  let (header, data) = parse(data, 200)?;
  Ok(Mid0200 { header, data })
}
