use crate::openprotocol::core::{Mid, MidHeader, mid_parse_header};

/// 5.3.5 MID 0014 Parameter set selected subscribe
#[derive(Debug, Default)]
pub struct Mid0014 {
  pub header: MidHeader,
}

impl Mid for Mid0014 {
  fn str(&self) -> String {
    self.header.str()
  }
}

pub fn mid_parse_0014(data: &str) -> Result<Mid0014, String> {
  let m14 = Mid0014 { header: mid_parse_header(data)? };
  if m14.header.mid != 14 {
    return Err(format!("Unexpected mid {} when parsing for mid 14", m14.header.mid));
  }
  Ok(m14)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0014_parses_header_only_message() {
    let m14 = mid_parse_0014("00200014001000000000").unwrap();
    assert_eq!(m14.str(), "00200014001000000000");
  }
}
