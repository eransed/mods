use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

const MF_0015_PARAMETER_SET_ID: MidField = MidField { name: "parameter_set_id", rng: 20..23 };
const MF_0015_DATE: MidField = MidField { name: "last_change", rng: 23..42 };

/// 5.3.6 MID 0015 Parameter set selected
#[derive(Debug, Default)]
pub struct Mid0015 {
  pub header: MidHeader,
  pub parameter_set_id: u16,
  pub last_change: String,
  pub revision2_data: String,
}

impl Mid for Mid0015 {
  fn str(&self) -> String {
    let mut header = self.header;
    header.len = if header.rev == 1 { 42 } else { 141 };
    let mut s = format!("{}{:03}", header.str(), self.parameter_set_id);
    if header.rev == 1 {
      s.push_str(&format!("{:<19}", self.last_change));
    } else {
      s.push_str(&self.revision2_data);
    }
    s
  }
}

pub fn mid_parse_0015(data: &str) -> Result<Mid0015, String> {
  let header = mid_parse_header(data)?;
  if header.mid != 15 {
    return Err(format!("Unexpected mid {} when parsing for mid 15", header.mid));
  }
  if !(1..=2).contains(&header.rev) {
    return Err(format!("Unsupported revision {} when parsing mid 15", header.rev));
  }
  let parameter_set_id = field_parse::<u16>(MF_0015_PARAMETER_SET_ID, data)?;
  let (last_change, revision2_data) = if header.rev == 1 {
    (field_parse::<String>(MF_0015_DATE, data)?, String::new())
  } else {
    (String::new(), data.get(23..).unwrap_or_default().to_string())
  };
  Ok(Mid0015 { header, parameter_set_id, last_change, revision2_data })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0015_rev1_parses_id_and_date() {
    let data = format!(
      "{}0122026-01-01:00:00:00",
      MidHeader { len: 42, mid: 15, rev: 1, ..Default::default() }.str()
    );
    let m15 = mid_parse_0015(&data).unwrap();
    assert_eq!(m15.parameter_set_id, 12);
    assert_eq!(m15.last_change, "2026-01-01:00:00:00");
  }
}
