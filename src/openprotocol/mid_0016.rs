use crate::openprotocol::core::{Mid, MidHeader, mid_parse_header};

/// 5.3.7 MID 0016 Parameter set selected acknowledge
#[derive(Debug, Default)]
pub struct Mid0016 {
  pub header: MidHeader,
}

impl Mid for Mid0016 {
  fn str(&self) -> String {
    self.header.str()
  }
}

pub fn mid_parse_0016(data: &str) -> Result<Mid0016, String> {
  let m16 = Mid0016 { header: mid_parse_header(data)? };
  if m16.header.mid != 16 {
    return Err(format!("Unexpected mid {} when parsing for mid 16", m16.header.mid));
  }
  Ok(m16)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0016_parses_header_only_message() {
    let m16 = mid_parse_0016("00200016001000000000").unwrap();
    assert_eq!(m16.str(), "00200016001000000000");
  }
}
