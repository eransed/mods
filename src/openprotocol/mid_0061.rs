use crate::openprotocol::core::{Mid, MidField, MidHeader, field_parse, mid_parse_header};

fn field(name: &'static str, start: usize, end: usize) -> MidField<'static> {
  MidField { name, rng: start..end }
}

/// MID 0061 Last tightening result data.
///
/// Revision 1 fields are exposed by name. Later revision data is preserved in `data` so fields
/// added by revisions 2 through 10 are not discarded.
#[derive(Debug, Clone, Default)]
pub struct Mid0061 {
  pub header: MidHeader,
  pub cell_id: String,
  pub channel_id: String,
  pub torque_controller_name: String,
  pub vin_number: String,
  pub job_id: String,
  pub parameter_set_id: String,
  pub batch_size: String,
  pub batch_counter: String,
  pub tightening_status: String,
  pub torque_status: String,
  pub angle_status: String,
  pub torque_min_limit: String,
  pub torque_max_limit: String,
  pub torque_final_target: String,
  pub torque: String,
  pub angle_min: String,
  pub angle_max: String,
  pub final_angle_target: String,
  pub angle: String,
  pub timestamp: String,
  pub parameter_set_last_change: String,
  pub batch_status: String,
  pub tightening_id: String,
  pub data: String,
}

impl Mid for Mid0061 {
  fn str(&self) -> String {
    let mut header = self.header;
    let data = if self.data.is_empty() {
      format!(
        "01{}02{}03{}04{}05{}06{}07{}08{}09{}10{}11{}12{}13{}14{}15{}16{}17{}18{}19{}20{}21{}22{}23{}",
        self.cell_id,
        self.channel_id,
        self.torque_controller_name,
        self.vin_number,
        self.job_id,
        self.parameter_set_id,
        self.batch_size,
        self.batch_counter,
        self.tightening_status,
        self.torque_status,
        self.angle_status,
        self.torque_min_limit,
        self.torque_max_limit,
        self.torque_final_target,
        self.torque,
        self.angle_min,
        self.angle_max,
        self.final_angle_target,
        self.angle,
        self.timestamp,
        self.parameter_set_last_change,
        self.batch_status,
        self.tightening_id,
      )
    } else {
      self.data.clone()
    };
    header.len = (20 + data.len()) as u16;
    format!("{}{}", header.str(), data)
  }
}

pub fn mid_parse_0061(data: &str) -> Result<Mid0061, String> {
  let header = mid_parse_header(data)?;
  if header.mid != 61 {
    return Err(format!("Unexpected mid {} when parsing for mid 61", header.mid));
  }
  let parsed = Mid0061 {
    header,
    cell_id: field_parse(field("cell_id", 22, 26), data)?,
    channel_id: field_parse(field("channel_id", 28, 30), data)?,
    torque_controller_name: field_parse(field("torque_controller_name", 32, 57), data)?,
    vin_number: field_parse(field("vin_number", 59, 84), data)?,
    job_id: field_parse(field("job_id", 86, 88), data)?,
    parameter_set_id: field_parse(field("parameter_set_id", 90, 93), data)?,
    batch_size: field_parse(field("batch_size", 95, 99), data)?,
    batch_counter: field_parse(field("batch_counter", 101, 105), data)?,
    tightening_status: field_parse(field("tightening_status", 107, 108), data)?,
    torque_status: field_parse(field("torque_status", 110, 111), data)?,
    angle_status: field_parse(field("angle_status", 113, 114), data)?,
    torque_min_limit: field_parse(field("torque_min_limit", 116, 122), data)?,
    torque_max_limit: field_parse(field("torque_max_limit", 124, 130), data)?,
    torque_final_target: field_parse(field("torque_final_target", 132, 138), data)?,
    torque: field_parse(field("torque", 140, 146), data)?,
    angle_min: field_parse(field("angle_min", 148, 153), data)?,
    angle_max: field_parse(field("angle_max", 155, 160), data)?,
    final_angle_target: field_parse(field("final_angle_target", 162, 167), data)?,
    angle: field_parse(field("angle", 169, 174), data)?,
    timestamp: field_parse(field("timestamp", 176, 195), data)?,
    parameter_set_last_change: field_parse(field("parameter_set_last_change", 197, 216), data)?,
    batch_status: field_parse(field("batch_status", 218, 219), data)?,
    tightening_id: field_parse(field("tightening_id", 221, 231), data)?,
    data: data.get(20..).unwrap_or_default().to_string(),
  };
  Ok(parsed)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_revision_one_fields() {
    let mut payload = vec![b' '; 211];
    let mut set = |start: usize, value: &str| {
      let offset = start - 20;
      payload[offset..offset + value.len()].copy_from_slice(value.as_bytes());
    };
    set(22, "0001");
    set(28, "01");
    set(32, &format!("{:<25}", "CTRL"));
    set(59, &format!("{:<25}", "VIN"));
    set(86, "01");
    set(90, "001");
    set(95, "0001");
    set(101, "0000");
    set(107, "1");
    set(110, "1");
    set(113, "1");
    set(116, "000100");
    set(124, "000200");
    set(132, "000150");
    set(140, "000125");
    set(148, "00010");
    set(155, "00020");
    set(162, "00015");
    set(169, "00010");
    set(176, "2026-01-01:00:00:00");
    set(197, "2026-01-01:00:00:00");
    set(218, "1");
    set(221, "0000000001");
    let payload = String::from_utf8(payload).unwrap();
    let message =
      format!("{}{}", MidHeader { len: 231, mid: 61, rev: 1, ..Default::default() }.str(), payload);
    let parsed = mid_parse_0061(&message).unwrap();
    assert_eq!(parsed.cell_id, "0001");
    assert_eq!(parsed.parameter_set_id, "001");
    assert_eq!(parsed.tightening_id, "0000000001");
  }
}
