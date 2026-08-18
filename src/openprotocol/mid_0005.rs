use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

const MF_0005_MID_NUMBER: MidField = MidField { name: "mid_number", rng: 20..24 };

/// 5.2.5 MID 0005 Application Communication positive acknowledge
///
/// This message is used by the controller to confirm that the latest command, request or
/// subscription sent by the integrator was accepted.
///
/// Message sent by: Controller
///
/// Answer: None
#[derive(Debug, Copy, Clone, Default)]
pub struct Mid0005 {
  pub header: MidHeader,
  /// MID number accepted.
  pub mid_number: u16,
}

impl Mid for Mid0005 {
  fn str(&self) -> String {
    let mut s = self.header.str();
    s.push_str(format!("{:04}", self.mid_number).as_str());
    s
  }
}

pub fn mid_parse_0005(data: &str) -> Result<Mid0005, String> {
  let mut m5 = Mid0005 { header: mid_parse_header(data)?, mid_number: 0 };

  if m5.header.mid != 5 {
    return Err(format!("Unexpected mid {} when parsing for mid 5", m5.header.mid));
  }

  m5.mid_number = field_parse::<u16>(MF_0005_MID_NUMBER, data)?;
  Ok(m5)
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use crate::openprotocol::core::MidName;

  use super::*;

  #[test]
  fn mid_0005_name() {
    assert_eq!(
      MidName::from_repr(5).unwrap(),
      MidName::from_str("Application Communication positive acknowledge").unwrap()
    );
  }

  #[test]
  fn mid_0005_parse_valid_mid_0005() {
    let m5 = mid_parse_0005("002400050010000000001234").unwrap();
    assert_eq!(m5.header.mid, 5);
    assert_eq!(m5.header.rev, 1);
    assert_eq!(m5.header.len, 24);
    assert_eq!(m5.mid_number, 1234);
  }

  #[test]
  fn mid_0005_serialize() {
    let m5 = Mid0005 {
      header: MidHeader { len: 24, mid: 5, rev: 1, ..Default::default() },
      mid_number: 1234,
    };
    assert_eq!(m5.str(), "002400050010000000001234");
  }

  #[test]
  fn mid_0005_parse_rejects_other_mid() {
    assert!(mid_parse_0005("002400040010000000001234").is_err());
  }

  #[test]
  fn mid_0005_parse_rejects_invalid_mid_number() {
    assert!(mid_parse_0005("00240005001000000000abcd").is_err());
  }
}
