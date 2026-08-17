use std::{fmt::Debug, ops::Range, str::FromStr, u16};

use tracing::{error, warn};

pub trait Mid<T> {
  fn new() -> T;
  fn name(self) -> String;
}

pub struct MidField<'a> {
  pub name: &'a str,
  pub rng: Range<usize>,
}

const MF_LEN: MidField = MidField { name: "len", rng: 0..4 };

const MF_MID: MidField = MidField { name: "mid", rng: 4..8 };

const MF_REV: MidField = MidField { name: "rev", rng: 8..11 };

const MF_NO_ACK_FLAG: MidField = MidField { name: "no_ack_flag", rng: 11..12 };

const MF_STATION_ID: MidField = MidField { name: "station_id", rng: 12..14 };

const MF_SPINDLE_ID: MidField = MidField { name: "spindle_id", rng: 14..16 };

const MF_SEQUENCE_NUMBER: MidField = MidField { name: "sequence_number", rng: 16..18 };

const MF_NUMBER_OF_MESSAGE_PARTS: MidField =
  MidField { name: "number_of_message_parts", rng: 18..19 };

const MF_MESSAGE_PART_NUMBER: MidField = MidField { name: "message_part_number", rng: 19..20 };

#[derive(Debug, Copy, Clone, Default)]
pub struct MidHeader {
  /// 1-4
  ///
  /// The length is the length of the header plus the data field
  /// excluding the NUL termination.
  /// The header always includes information about the length
  /// of the message. The length is represented by four ASCII
  /// digits (‘0’…’9’) specifying a range of 0000 to 9999.
  /// When using the message linking functionality the length
  /// represents the length of each message part number.
  /// When having one ASCII part followed by an binary part
  /// the length is the total length of the message.
  pub len: u16,

  /// 5-8
  ///
  /// The MID is four bytes long and is specified by four ASCII
  /// digits (‘0’…’9’). The MID describes how to interpret the
  /// message.
  pub mid: u16,

  /// 9-11
  ///
  /// The revision of the MID is specified by three ASCII digits
  /// (‘0’…’9’).
  /// The MID Revision is unique per MID and is used in case
  /// different versions are available for the same MID. Using
  /// the revision number the integrator can subscribe or ask
  /// for different versions of the same MID. By default the
  /// MID revision number is three spaces long.
  /// If the initial MID Revision (revision 1) is required there is
  /// three different ways to get it, either send three spaces or
  /// 000 or 001.
  pub rev: u16,

  pub no_ack_flag: u8,
  pub station_id: u8,
  pub spindle_id: u8,
  pub sequence_number: u8,
  pub number_of_message_parts: u8,
  pub message_part_number: u8,
}

// pub fn mid_header_str(h: MidHeader) -> String {
//   format!(
//     "{:04}{:04}{:03}{:01}{:02}{:02}{:02}{:01}{:01}",
//     h.len,
//     h.mid,
//     h.rev,
//     h.no_ack_flag,
//     h.station_id,
//     h.spindle_id,
//     h.sequence_number,
//     h.number_of_message_parts,
//     h.message_part_number,
//   )
// }

pub fn mid_header_str(h: MidHeader) -> String {
  let lw = MF_LEN.rng.end - MF_LEN.rng.start;
  let mw = MF_MID.rng.end - MF_MID.rng.start;
  let rw = MF_REV.rng.end - MF_REV.rng.start;
  let noaw = MF_NO_ACK_FLAG.rng.end - MF_NO_ACK_FLAG.rng.start;
  let siw = MF_STATION_ID.rng.end - MF_STATION_ID.rng.start;
  let spw = MF_SPINDLE_ID.rng.end - MF_SPINDLE_ID.rng.start;
  let snw = MF_SEQUENCE_NUMBER.rng.end - MF_SEQUENCE_NUMBER.rng.start;
  let nompw = MF_NUMBER_OF_MESSAGE_PARTS.rng.end - MF_NUMBER_OF_MESSAGE_PARTS.rng.start;
  let mpnw = MF_MESSAGE_PART_NUMBER.rng.end - MF_MESSAGE_PART_NUMBER.rng.start;
  format!(
    "{:0lw$}{:0mw$}{:0rw$}{:0noaw$}{:0siw$}{:0spw$}{:0snw$}{:0nompw$}{:0mpnw$}",
    h.len,
    h.mid,
    h.rev,
    h.no_ack_flag,
    h.station_id,
    h.spindle_id,
    h.sequence_number,
    h.number_of_message_parts,
    h.message_part_number
  )
}

pub fn field_parse<T: std::str::FromStr>(field: MidField, data: &str) -> Result<T, String>
where
  <T as FromStr>::Err: std::fmt::Display,
{
  match data.get(field.rng.clone()) {
    Some(v) => match v.parse::<T>() {
      Ok(l) => return Ok(l),
      Err(e) => {
        let d = field.rng.end - field.rng.start;
        if field.rng.start != field.rng.end && d > 1 {
          return Err(format!(
            "Could not parse field '{}' from slice '{}' of len {}: {}: parsing the message:\n\n   '{}'\n{}{}{}{}\n",
            field.name,
            v,
            v.len(),
            e,
            data,
            " ".repeat(field.rng.start + 3 + 1),
            "^",
            "^".repeat(d - 2),
            "^"
          ));
        } else {
          return Err(format!(
            "Could not parse field '{}' from slice '{}' of len {}: {}: parsing the message:\n\n   '{}'\n{}{}\n",
            field.name,
            v,
            v.len(),
            e,
            data,
            " ".repeat(field.rng.start + 3 + 1),
            "^"
          ));
        }
      }
    },
    None => {
      return Err(format!(
        "Field '{}' at range {:?} not found in mid '{}' of length {}",
        field.name,
        field.rng,
        data,
        data.len()
      ));
    }
  }
}

pub fn get_mid(data: &str) -> Result<u16, String> {
  return field_parse::<u16>(MF_MID, data);
}

pub fn mid_parse_header(raw_mid: &str) -> Result<MidHeader, String> {
  let mut header = MidHeader::new();
  let l = raw_mid.len();
  if l < 20 {
    println!("Mid '{}' to short, len = {}", raw_mid, l);
    return Err(format!("Mid '{}' to short, len = {}", raw_mid, l));
  }

  match field_parse::<u16>(MF_LEN, raw_mid) {
    Ok(v) => header.len = v,
    Err(e) => error!("{}", e),
  }

  println!("Parsing mid len to {} from raw mid '{}'", header.len, raw_mid);

  if raw_mid.len() != header.len as usize {
    warn!("Reported len {} not equal to actual message len {}", header.len, raw_mid.len());
  }

  match field_parse::<u16>(MF_MID, raw_mid) {
    Ok(v) => header.mid = v,
    Err(e) => error!("{}", e),
  }

  match field_parse::<u16>(MF_REV, raw_mid) {
    Ok(v) => header.rev = v,
    Err(e) => error!("{}", e),
  }

  if header.len < 20 {
    return Err(format!("Invalid length '{}' when parsing the mid header", header.len));
  }

  if header.mid < 1 || header.mid > 9999 {
    return Err(format!("Invalid mid '{}' when parsing the mid header", header.mid));
  }

  if header.rev < 1 || header.rev > 9999 {
    return Err(format!("Invalid revision '{}' when parsing the mid header", header.rev));
  }

  match field_parse::<u8>(MF_NO_ACK_FLAG, raw_mid) {
    Ok(v) => header.no_ack_flag = v,
    Err(e) => error!("{}", e),
  }

  match field_parse::<u8>(MF_STATION_ID, raw_mid) {
    Ok(v) => header.station_id = v,
    Err(e) => error!("{}", e),
  }

  match field_parse::<u8>(MF_SPINDLE_ID, raw_mid) {
    Ok(v) => header.spindle_id = v,
    Err(e) => error!("{}", e),
  }

  match field_parse::<u8>(MF_SEQUENCE_NUMBER, raw_mid) {
    Ok(v) => header.sequence_number = v,
    Err(e) => error!("{}", e),
  }

  match field_parse::<u8>(MF_NUMBER_OF_MESSAGE_PARTS, raw_mid) {
    Ok(v) => header.number_of_message_parts = v,
    Err(e) => error!("{}", e),
  }

  match field_parse::<u8>(MF_MESSAGE_PART_NUMBER, raw_mid) {
    Ok(v) => header.message_part_number = v,
    Err(e) => error!("{}", e),
  }
  Ok(header)
}

impl Mid<MidHeader> for MidHeader {
  fn new() -> MidHeader {
    MidHeader {
      len: 0,
      mid: 0,
      rev: 0,
      no_ack_flag: 0,
      station_id: 0,
      spindle_id: 0,
      sequence_number: 0,
      number_of_message_parts: 0,
      message_part_number: 0,
    }
  }

  fn name(self) -> String {
    String::from("Raw MID header")
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn mid_header_valid_mid42_rev1_zeros_ok() {
    let _ = mid_parse_header("00200042001000000000").unwrap();
  }

  #[test]
  fn mid_header_valid_mid42_rev1_spaces_ok() {
    let _ = mid_parse_header("00200042001         ").unwrap();
  }

  #[test]
  fn mid_header_str_1() {
    let h = MidHeader::new();
    assert_eq!(mid_header_str(h), "00000000000000000000");
  }

  #[test]
  fn mid_header_str_2() {
    let mut h = MidHeader::new();
    h.len = 20;
    assert_eq!(mid_header_str(h), "00200000000000000000");
  }

  #[test]
  fn mid_header_str_3() {
    let mut h = MidHeader::new();
    h.len = 20;
    h.mid = 1;
    assert_eq!(mid_header_str(h), "00200001000000000000");
  }

  #[test]
  fn mid_header_str_4() {
    let mut h = MidHeader::new();
    h.len = 1234;
    h.mid = 1;
    h.rev = 4;
    h.no_ack_flag = 1;
    assert_eq!(mid_header_str(h), "12340001004100000000");
  }

  #[test]
  fn mid_header_str_5() {
    let mut h = MidHeader::new();
    h.len = 1111;
    h.mid = 2222;
    h.rev = 333;
    h.no_ack_flag = 4;
    h.station_id = 55;
    h.spindle_id = 66;
    h.sequence_number = 77;
    h.number_of_message_parts = 8;
    h.message_part_number = 9;
    assert_eq!(mid_header_str(h), "11112222333455667789");
  }

  // Error tests

  #[test]
  #[should_panic]
  fn mid_header_empty_string_error() {
    let _ = mid_parse_header("").unwrap();
  }

  #[test]
  #[should_panic]
  fn mid_header_20_zeros_error() {
    let _ = mid_parse_header("00000000000000000000").unwrap();
  }

  #[test]
  #[should_panic]
  fn mid_header_len_20_correct_len_then_zeros_invalid_mid_error() {
    let _ = mid_parse_header("00200000000000000000").unwrap();
  }

  #[test]
  #[should_panic]
  fn mid_header_len_20_correct_len_then_numbers_invalid_mid_error() {
    let _ = mid_parse_header("00200000987654321000").unwrap();
  }

  #[test]
  #[should_panic]
  fn mid_header_len_20_correct_43_zero_revision_then_spaces_error() {
    let _ = mid_parse_header("00200043000         ").unwrap();
  }

  #[test]
  #[should_panic]
  fn mid_header_too_short_and_invalid_1_error() {
    let _ = mid_parse_header("ajkl;hdfj").unwrap();
  }

  #[test]
  #[should_panic]
  fn mid_header_too_short_and_invalid_2_error() {
    let _ = mid_parse_header("00000000alsdfj").unwrap();
  }

  #[test]
  #[should_panic]
  fn mid_header_invalid_1_error() {
    let _ =
      mid_parse_header("sl;dfkjashdgasklfdj;adf;aklsdfhghsldkfjasbcvjldsjflasjdflaksdfa;lksdjf")
        .unwrap();
  }
}
