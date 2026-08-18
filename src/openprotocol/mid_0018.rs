use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

const MF_0018_PARAMETER_SET_ID: MidField = MidField { name: "parameter_set_id", rng: 20..23 };

/// 5.3.9 MID 0018 Select Parameter set
#[derive(Debug, Default)]
pub struct Mid0018 {
  pub header: MidHeader,
  pub parameter_set_id: u16,
}

impl Mid for Mid0018 {
  fn str(&self) -> String {
    let mut s = self.header.str();
    s.push_str(&format!("{:03}", self.parameter_set_id));
    s
  }
}

pub fn mid_parse_0018(data: &str) -> Result<Mid0018, String> {
  let mut m18 = Mid0018 { header: mid_parse_header(data)?, parameter_set_id: 0 };
  if m18.header.mid != 18 {
    return Err(format!("Unexpected mid {} when parsing for mid 18", m18.header.mid));
  }
  m18.parameter_set_id = field_parse::<u16>(MF_0018_PARAMETER_SET_ID, data)?;
  Ok(m18)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0018_parses_and_serializes() {
    let m18 = mid_parse_0018("00230018001000000000123").unwrap();
    assert_eq!(m18.parameter_set_id, 123);
    assert_eq!(m18.str(), "00230018001000000000123");
  }
}
