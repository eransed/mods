use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0214 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0214 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0214(data: &str) -> Result<Mid0214, String> {
  let (header, data) = parse(data, 214)?;
  Ok(Mid0214 { header, data })
}
