use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

fn field(name: &'static str, start: usize, width: usize) -> MidField<'static> {
  MidField { name, rng: start..start + width }
}

/// 5.3.4 MID 0013 Parameter set data upload reply.
#[derive(Debug, Default)]
pub struct Mid0013 {
  pub header: MidHeader,
  pub parameter_set_id: String,
  pub parameter_set_name: String,
  pub rotation_direction: String,
  pub batch_size: String,
  pub torque_min: String,
  pub torque_max: String,
  pub torque_final_target: String,
  pub angle_min: String,
  pub angle_max: String,
  pub final_angle_target: String,
  pub first_target: String,
  pub start_final_angle: String,
  pub last_change: String,
  pub pset_file_version: String,
  pub parameter_set_data: String,
}

impl Mid for Mid0013 {
  fn str(&self) -> String {
    let rev = self.header.rev;
    let mut header = self.header;
    header.len = match rev {
      1 => 104,
      2 => 120,
      3 | 4 => 28 + self.parameter_set_data.len() as u16,
      5 => 141,
      _ => self.header.len,
    };
    let mut s = header.str();
    if matches!(rev, 3 | 4) {
      s.push_str(&format!("{:<8}", self.pset_file_version));
      s.push_str(&self.parameter_set_data);
      return s;
    }
    s.push_str(&format!("01{:03}", self.parameter_set_id));
    s.push_str(&format!("02{:<25}", self.parameter_set_name));
    s.push_str(&format!("03{}", self.rotation_direction));
    s.push_str(&format!("04{:02}", self.batch_size));
    s.push_str(&format!("05{:06}", self.torque_min));
    s.push_str(&format!("06{:06}", self.torque_max));
    s.push_str(&format!("07{:06}", self.torque_final_target));
    s.push_str(&format!("08{:05}", self.angle_min));
    s.push_str(&format!("09{:05}", self.angle_max));
    s.push_str(&format!("10{:05}", self.final_angle_target));
    if rev >= 2 {
      s.push_str(&format!("11{:06}", self.first_target));
      s.push_str(&format!("12{:06}", self.start_final_angle));
    }
    if rev == 5 {
      s.push_str(&format!("13{:<19}", self.last_change));
    }
    s
  }
}

pub fn mid_parse_0013(data: &str) -> Result<Mid0013, String> {
  let header = mid_parse_header(data)?;
  if header.mid != 13 {
    return Err(format!("Unexpected mid {} when parsing for mid 13", header.mid));
  }
  if !(1..=5).contains(&header.rev) {
    return Err(format!("Unsupported revision {} when parsing mid 13", header.rev));
  }
  let mut m13 = Mid0013 { header, ..Default::default() };
  if matches!(header.rev, 3 | 4) {
    m13.pset_file_version = field_parse::<String>(field("pset_file_version", 20, 8), data)?;
    m13.parameter_set_data = data.get(28..).unwrap_or_default().to_string();
    return Ok(m13);
  }
  m13.parameter_set_id = field_parse::<String>(field("parameter_set_id", 22, 3), data)?;
  m13.parameter_set_name = field_parse::<String>(field("parameter_set_name", 27, 25), data)?;
  m13.rotation_direction = field_parse::<String>(field("rotation_direction", 54, 1), data)?;
  m13.batch_size = field_parse::<String>(field("batch_size", 57, 2), data)?;
  m13.torque_min = field_parse::<String>(field("torque_min", 61, 6), data)?;
  m13.torque_max = field_parse::<String>(field("torque_max", 69, 6), data)?;
  m13.torque_final_target = field_parse::<String>(field("torque_final_target", 77, 6), data)?;
  m13.angle_min = field_parse::<String>(field("angle_min", 85, 5), data)?;
  m13.angle_max = field_parse::<String>(field("angle_max", 92, 5), data)?;
  m13.final_angle_target = field_parse::<String>(field("final_angle_target", 99, 5), data)?;
  if header.rev >= 2 {
    m13.first_target = field_parse::<String>(field("first_target", 106, 6), data)?;
    m13.start_final_angle = field_parse::<String>(field("start_final_angle", 114, 6), data)?;
  }
  if header.rev == 5 {
    m13.last_change = field_parse::<String>(field("last_change", 122, 19), data)?;
  }
  Ok(m13)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0013_rev1_parses_fixed_fields() {
    let data = format!(
      "{}01{:03}02{:<25}03{}04{}05{}06{}07{}08{}09{}10{}",
      MidHeader { len: 104, mid: 13, rev: 1, ..Default::default() }.str(),
      "001",
      "Airbag 1",
      "1",
      "01",
      "000010",
      "000060",
      "000070",
      "00080",
      "00090",
      "00010"
    );
    let m13 = mid_parse_0013(&data).unwrap();
    assert_eq!(m13.parameter_set_id, "001");
    assert_eq!(m13.parameter_set_name, "Airbag 1                 ");
    assert_eq!(m13.torque_min, "000010");
  }

  #[test]
  fn mid_0013_rev3_preserves_variable_data() {
    let m13 = mid_parse_0013("0033001300300000000000000000RAW-DATA").unwrap();
    assert_eq!(m13.pset_file_version, "00000000");
    assert_eq!(m13.parameter_set_data, "RAW-DATA");
  }
}
