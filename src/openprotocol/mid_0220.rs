use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0220 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0220 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0220(data: &str) -> Result<Mid0220, String> {
  let (header, data) = parse(data, 220)?;
  Ok(Mid0220 { header, data })
}
