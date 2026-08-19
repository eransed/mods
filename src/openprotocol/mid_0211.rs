use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

const MF_DIGITAL_INPUT_1: MidField = MidField { name: "digital_input_1", rng: 20..21 };
const MF_DIGITAL_INPUT_2: MidField = MidField { name: "digital_input_2", rng: 21..22 };
const MF_DIGITAL_INPUT_3: MidField = MidField { name: "digital_input_3", rng: 22..23 };
const MF_DIGITAL_INPUT_4: MidField = MidField { name: "digital_input_4", rng: 23..24 };
const MF_DIGITAL_INPUT_5: MidField = MidField { name: "digital_input_5", rng: 24..25 };
const MF_DIGITAL_INPUT_6: MidField = MidField { name: "digital_input_6", rng: 25..26 };
const MF_DIGITAL_INPUT_7: MidField = MidField { name: "digital_input_7", rng: 26..27 };
const MF_DIGITAL_INPUT_8: MidField = MidField { name: "digital_input_8", rng: 27..28 };

#[derive(Debug, Clone, Default)]
pub struct Mid0211 {
  pub header: MidHeader,
  pub digital_input_1: u8,
  pub digital_input_2: u8,
  pub digital_input_3: u8,
  pub digital_input_4: u8,
  pub digital_input_5: u8,
  pub digital_input_6: u8,
  pub digital_input_7: u8,
  pub digital_input_8: u8,
}
impl Mid for Mid0211 {
  fn str(&self) -> String {
    let mut header = self.header;
    header.len = 28;
    format!(
      "{}{}{}{}{}{}{}{}{}",
      header.str(),
      self.digital_input_1,
      self.digital_input_2,
      self.digital_input_3,
      self.digital_input_4,
      self.digital_input_5,
      self.digital_input_6,
      self.digital_input_7,
      self.digital_input_8
    )
  }
}
pub fn mid_parse_0211(data: &str) -> Result<Mid0211, String> {
  let header = mid_parse_header(data)?;
  if header.mid != 211 {
    return Err(format!("Unexpected mid {} when parsing for mid 211", header.mid));
  }
  let digital_input_1 = field_parse::<u8>(MF_DIGITAL_INPUT_1, data)?;
  let digital_input_2 = field_parse::<u8>(MF_DIGITAL_INPUT_2, data)?;
  let digital_input_3 = field_parse::<u8>(MF_DIGITAL_INPUT_3, data)?;
  let digital_input_4 = field_parse::<u8>(MF_DIGITAL_INPUT_4, data)?;
  let digital_input_5 = field_parse::<u8>(MF_DIGITAL_INPUT_5, data)?;
  let digital_input_6 = field_parse::<u8>(MF_DIGITAL_INPUT_6, data)?;
  let digital_input_7 = field_parse::<u8>(MF_DIGITAL_INPUT_7, data)?;
  let digital_input_8 = field_parse::<u8>(MF_DIGITAL_INPUT_8, data)?;
  if [
    digital_input_1,
    digital_input_2,
    digital_input_3,
    digital_input_4,
    digital_input_5,
    digital_input_6,
    digital_input_7,
    digital_input_8,
  ]
  .iter()
  .any(|status| *status > 1)
  {
    return Err("Digital input status must be 0 or 1".to_string());
  }
  Ok(Mid0211 {
    header,
    digital_input_1,
    digital_input_2,
    digital_input_3,
    digital_input_4,
    digital_input_5,
    digital_input_6,
    digital_input_7,
    digital_input_8,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_all_digital_input_statuses() {
    let message = "0028021100100000000001100111";
    let parsed = mid_parse_0211(message).unwrap();
    assert_eq!(parsed.digital_input_1, 0);
    assert_eq!(parsed.digital_input_2, 1);
    assert_eq!(parsed.digital_input_3, 1);
    assert_eq!(parsed.digital_input_4, 0);
    assert_eq!(parsed.digital_input_5, 0);
    assert_eq!(parsed.digital_input_6, 1);
    assert_eq!(parsed.digital_input_7, 1);
    assert_eq!(parsed.digital_input_8, 1);
  }

  #[test]
  fn serializes_all_digital_input_statuses() {
    let message = "0028021100100000000001100111";
    let parsed = mid_parse_0211(message).unwrap();
    assert_eq!(parsed.str(), message);
  }

  #[test]
  fn rejects_invalid_digital_input_status() {
    assert!(mid_parse_0211("0028021100100000000020110011").is_err());
  }
}
