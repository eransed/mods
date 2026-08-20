use crate::openprotocol::core::{Mid, MidHeader, parse, serialize};
#[derive(Debug, Clone, Default)]
pub struct Mid9999 {
  pub header: MidHeader,
  pub data: String,
}
impl Mid for Mid9999 {
  fn str(&self) -> String {
    serialize(self.header, &self.data)
  }
}
pub fn mid_parse_9999(data: &str) -> Result<Mid9999, String> {
  let (header, data) = parse(data, 9999)?;
  Ok(Mid9999 { header, data })
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_9999_parse_valid_mid_9999_blank_revision() {
    let m9999 = mid_parse_9999("00209999    0000    ").unwrap();
    assert_eq!(m9999.header.mid, 9999);
    assert_eq!(m9999.header.len, 20);
    assert_eq!(m9999.header.rev, 0);
  }
}