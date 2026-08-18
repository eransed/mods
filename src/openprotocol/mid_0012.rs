use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

const MF_0012_PARAMETER_SET_ID: MidField = MidField { name: "parameter_set_id", rng: 20..23 };
const MF_0012_PSET_FILE_VERSION: MidField = MidField { name: "pset_file_version", rng: 23..31 };

/// 5.3.3 MID 0012 Parameter set data upload request
#[derive(Debug, Default)]
pub struct Mid0012 {
  pub header: MidHeader,
  pub parameter_set_id: Option<u16>,
  pub pset_file_version: Option<String>,
}

impl Mid for Mid0012 {
  fn str(&self) -> String {
    let mut header = self.header;
    if matches!(header.rev, 3 | 4) {
      header.len = 31;
    } else {
      header.len = 23;
    }
    let mut s = header.str();
    if matches!(header.rev, 3 | 4) {
      s.push_str("000");
      s.push_str(self.pset_file_version.as_deref().unwrap_or("00000000"));
    } else {
      s.push_str(&format!("{:03}", self.parameter_set_id.unwrap_or_default()));
    }
    s
  }
}

pub fn mid_parse_0012(data: &str) -> Result<Mid0012, String> {
  let header = mid_parse_header(data)?;
  if header.mid != 12 {
    return Err(format!("Unexpected mid {} when parsing for mid 12", header.mid));
  }
  let (parameter_set_id, pset_file_version) = if matches!(header.rev, 3 | 4) {
    (None, Some(field_parse::<String>(MF_0012_PSET_FILE_VERSION, data)?))
  } else if matches!(header.rev, 1 | 2 | 5) {
    (Some(field_parse::<u16>(MF_0012_PARAMETER_SET_ID, data)?), None)
  } else {
    return Err(format!("Unsupported revision {} when parsing mid 12", header.rev));
  };
  Ok(Mid0012 { header, parameter_set_id, pset_file_version })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0012_rev1_parses_parameter_set_id() {
    let m12 = mid_parse_0012("00230012001000000000123").unwrap();
    assert_eq!(m12.parameter_set_id, Some(123));
  }

  #[test]
  fn mid_0012_rev3_parses_file_version() {
    let m12 = mid_parse_0012("0031001200300000000000000000000").unwrap();
    assert_eq!(m12.pset_file_version.as_deref(), Some("00000000"));
  }
}
