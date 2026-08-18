use crate::openprotocol::core::{Mid, MidHeader, mid_parse_header};

/// 5.3.8 MID 0017 Parameter set selected unsubscribe
#[derive(Debug, Default)]
pub struct Mid0017 {
  pub header: MidHeader,
}

impl Mid for Mid0017 {
  fn str(&self) -> String {
    self.header.str()
  }
}

pub fn mid_parse_0017(data: &str) -> Result<Mid0017, String> {
  let m17 = Mid0017 { header: mid_parse_header(data)? };
  if m17.header.mid != 17 {
    return Err(format!("Unexpected mid {} when parsing for mid 17", m17.header.mid));
  }
  Ok(m17)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0017_parses_header_only_message() {
    let m17 = mid_parse_0017("00200017001000000000").unwrap();
    assert_eq!(m17.str(), "00200017001000000000");
  }
}
