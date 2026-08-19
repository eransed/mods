use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0053 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0053 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0053(data: &str) -> Result<Mid0053, String> {
  let (header, data) = parse(data, 53)?;
  Ok(Mid0053 { header, data })
}
