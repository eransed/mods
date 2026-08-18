use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

const MF_0011_NUMBER_OF_PARAMETER_SETS: MidField =
  MidField { name: "number_of_parameter_sets", rng: 20..23 };

#[derive(Debug, Default, Clone)]
pub struct Mid0011ParameterSet {
  pub id: u16,
  pub stages: u8,
  pub program_type: String,
  pub last_change: String,
}

/// 5.3.2 MID 0011 Parameter set ID upload reply
#[derive(Debug, Default)]
pub struct Mid0011 {
  pub header: MidHeader,
  pub parameter_sets: Vec<Mid0011ParameterSet>,
}

impl Mid for Mid0011 {
  fn str(&self) -> String {
    let rev = self.header.rev;
    let mut header = self.header;
    let count = self.parameter_sets.len();
    header.len = match rev {
      1 => 23 + count as u16 * 3,
      2 => 23 + count as u16 * 5,
      3 => 23 + count as u16 * 9,
      4 => 23 + count as u16 * 28,
      _ => self.header.len,
    };

    let mut s = format!("{}{:03}", header.str(), count);
    for parameter_set in &self.parameter_sets {
      s.push_str(&format!("{:03}", parameter_set.id));
    }
    if rev >= 2 {
      for parameter_set in &self.parameter_sets {
        s.push_str(&format!("{:02}", parameter_set.stages));
      }
    }
    if rev >= 3 {
      for parameter_set in &self.parameter_sets {
        s.push_str(&format!("{:<4}", parameter_set.program_type));
      }
    }
    if rev >= 4 {
      for parameter_set in &self.parameter_sets {
        s.push_str(&format!("{:<19}", parameter_set.last_change));
      }
    }
    s
  }
}

pub fn mid_parse_0011(data: &str) -> Result<Mid0011, String> {
  let header = mid_parse_header(data)?;
  if header.mid != 11 {
    return Err(format!("Unexpected mid {} when parsing for mid 11", header.mid));
  }
  if !(1..=4).contains(&header.rev) {
    return Err(format!("Unsupported revision {} when parsing mid 11", header.rev));
  }

  let count = field_parse::<usize>(MF_0011_NUMBER_OF_PARAMETER_SETS, data)?;
  let mut parameter_sets = vec![Mid0011ParameterSet::default(); count];
  for (index, parameter_set) in parameter_sets.iter_mut().enumerate() {
    let start = 23 + index * 3;
    parameter_set.id =
      field_parse::<u16>(MidField { name: "parameter_set_id", rng: start..start + 3 }, data)?;
  }
  if header.rev >= 2 {
    for (index, parameter_set) in parameter_sets.iter_mut().enumerate() {
      let start = 23 + count * 3 + index * 2;
      parameter_set.stages =
        field_parse::<u8>(MidField { name: "stages", rng: start..start + 2 }, data)?;
    }
  }
  if header.rev >= 3 {
    for (index, parameter_set) in parameter_sets.iter_mut().enumerate() {
      let start = 23 + count * 5 + index * 4;
      parameter_set.program_type =
        field_parse::<String>(MidField { name: "program_type", rng: start..start + 4 }, data)?;
    }
  }
  if header.rev >= 4 {
    for (index, parameter_set) in parameter_sets.iter_mut().enumerate() {
      let start = 23 + count * 9 + index * 19;
      parameter_set.last_change =
        field_parse::<String>(MidField { name: "last_change", rng: start..start + 19 }, data)?;
    }
  }
  Ok(Mid0011 { header, parameter_sets })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mid_0011_rev1_parse_and_serialize() {
    let data =
      format!("{}002001002", MidHeader { len: 29, mid: 11, rev: 1, ..Default::default() }.str());
    let m11 = mid_parse_0011(&data).unwrap();
    assert_eq!(m11.parameter_sets.len(), 2);
    assert_eq!(m11.parameter_sets[0].id, 1);
    assert_eq!(m11.parameter_sets[1].id, 2);
    assert_eq!(m11.str(), data);
  }

  #[test]
  fn mid_0011_rev4_parses_repeated_fields() {
    let data = format!(
      "{}0020010020102PsetMset2026-01-01:00:00:002026-01-02:00:00:00",
      MidHeader { len: 79, mid: 11, rev: 4, ..Default::default() }.str()
    );
    let m11 = mid_parse_0011(&data).unwrap();
    assert_eq!(m11.parameter_sets[0].stages, 1);
    assert_eq!(m11.parameter_sets[1].program_type, "Mset");
    assert_eq!(m11.parameter_sets[1].last_change, "2026-01-02:00:00:00");
  }

  #[test]
  fn mid_0011_rejects_other_mid() {
    assert!(mid_parse_0011("00230010001000000000001").is_err());
  }
}
