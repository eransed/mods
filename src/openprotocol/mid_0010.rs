use crate::openprotocol::core::{Mid, MidHeader, mid_parse_header};

/// 5.3.1 MID 0010 Parameter set ID upload request
#[derive(Debug, Default)]
pub struct Mid0010 {
  pub header: MidHeader,
}

impl Mid for Mid0010 {
  fn str(&self) -> String {
    self.header.str()
  }
}

pub fn mid_parse_0010(data: &str) -> Result<Mid0010, String> {
  let m10 = Mid0010 { header: mid_parse_header(data)? };
  if m10.header.mid != 10 {
    return Err(format!("Unexpected mid {} when parsing for mid 10", m10.header.mid));
  }
  Ok(m10)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0010_parse_and_serialize() {
    let m10 = mid_parse_0010("00200010001000000000").unwrap();
    assert_eq!(m10.str(), "00200010001000000000");
  }

  #[test]
  fn mid_0010_rejects_other_mid() {
    assert!(mid_parse_0010("00200011001000000000").is_err());
  }
}
