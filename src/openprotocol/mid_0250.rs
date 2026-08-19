use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0250 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0250 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0250(data: &str) -> Result<Mid0250, String> {
  let (header, data) = parse(data, 250)?;
  Ok(Mid0250 { header, data })
}
