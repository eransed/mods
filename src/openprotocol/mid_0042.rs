use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid0042 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid0042 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_0042(data: &str) -> Result<Mid0042, String> {
  let (header, data) = parse(data, 42)?;
  Ok(Mid0042 { header, data })
}
