use crate::openprotocol::core::{Mid, MidHeader, mid_parse_header};

/// 5.2.3 MID 0003 Application Communication stop
/// 
/// This message disables the communication. The controller will stop to respond to any commands
/// except for MID 0001 Communication start after receiving this command.
/// 
/// Message sent by: Integrator
/// 
/// Answer: MID 0005 Command accepted
#[derive(Debug, Default)]
pub struct Mid0003 {
  pub header: MidHeader,
}

impl Mid for Mid0003 {
  fn str(&self) -> String {
    self.header.str()
  }
}

pub fn mid_parse_0003(data: &str) -> Result<Mid0003, String> {
  let m3 = Mid0003 { header: mid_parse_header(data)? };

  if m3.header.mid != 3 {
    return Err(format!("Unexpected mid {} when parsing for mid 3", m3.header.mid));
  }

  if m3.header.rev != 1 {
    return Err(format!(
      "Unsupported revision {} when parsing mid 3; only revision 1 is supported",
      m3.header.rev
    ));
  }

  Ok(m3)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0003_parse_valid_mid_0003() {
    let m3 = mid_parse_0003("00200003001000000000").unwrap();
    assert_eq!(m3.header.mid, 3);
    assert_eq!(m3.header.rev, 1);
    assert_eq!(m3.header.len, 20);
  }

  #[test]
  fn mid_0003_serialize() {
    let mut m3 = Mid0003::default();
    m3.header.len = 20;
    m3.header.mid = 3;
    m3.header.rev = 1;
    assert_eq!(m3.str(), "00200003001000000000");
  }

  #[test]
  fn mid_0003_parse_rejects_other_mid() {
    assert!(mid_parse_0003("00200002001000000000").is_err());
  }

  #[test]
  fn mid_0003_parse_rejects_other_revision() {
    assert!(mid_parse_0003("00200003002000000000").is_err());
  }
}
