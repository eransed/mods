use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0021 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0021 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0021(data: &str) -> Result<Mid0021, String> {
  let (header, data) = parse(data, 21)?;
  Ok(Mid0021 { header, data })
}
