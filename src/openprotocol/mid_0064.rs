use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0064 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0064 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0064(data: &str) -> Result<Mid0064, String> {
  let (header, data) = parse(data, 64)?;
  Ok(Mid0064 { header, data })
}
