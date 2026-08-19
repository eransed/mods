use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0070 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0070 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0070(data: &str) -> Result<Mid0070, String> {
  let (header, data) = parse(data, 70)?;
  Ok(Mid0070 { header, data })
}
