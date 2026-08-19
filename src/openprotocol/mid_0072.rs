use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0072 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0072 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0072(data: &str) -> Result<Mid0072, String> {
  let (header, data) = parse(data, 72)?;
  Ok(Mid0072 { header, data })
}
