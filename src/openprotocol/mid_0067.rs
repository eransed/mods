use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0067 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0067 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0067(data: &str) -> Result<Mid0067, String> {
  let (header, data) = parse(data, 67)?;
  Ok(Mid0067 { header, data })
}
