use std::{fmt::Debug, ops::Range, str::FromStr, u16};
use strum::{AsRefStr, EnumIter, EnumString, FromRepr, IntoStaticStr};

use tracing::{error, warn};

pub trait Mid {
  fn str(&self) -> String;
}

pub fn parse(data: &str, expected_mid: u16) -> Result<(MidHeader, String), String> {
  let header = mid_parse_header(data)?;
  if header.mid != expected_mid {
    return Err(format!("Unexpected mid {} when parsing for mid {}", header.mid, expected_mid));
  }
  Ok((header, data.get(20..).unwrap_or_default().to_string()))
}

pub fn serialize(header: MidHeader, data: &str) -> String {
  let mut header = header;
  header.len = (20 + data.len()) as u16;
  format!("{}{}", header.str(), data)
}

pub struct MidField<'a> {
  pub name: &'a str,
  pub rng: Range<usize>,
}

pub fn field_parse_with_default<T: std::str::FromStr>(field: MidField, data: &str, default: T) -> T
where
  <T as FromStr>::Err: std::fmt::Display,
{
  match field_parse::<T>(field, data) {
    Ok(v) => v,
    Err(_) => default,
  }
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

// Header

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

impl Mid for MidHeader {
  fn str(&self) -> String {
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
      self.len,
      self.mid,
      self.rev,
      self.no_ack_flag,
      self.station_id,
      self.spindle_id,
      self.sequence_number,
      self.number_of_message_parts,
      self.message_part_number
    )
  }
}

pub fn mid_parse_header(raw_mid: &str) -> Result<MidHeader, String> {
  let mut header = MidHeader::default();
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
    Err(e) => {
      error!("{}", e)
    }
  }

  header.rev = field_parse_with_default(MF_REV, raw_mid, 0);

  if header.len < 20 {
    return Err(format!("Invalid length '{}' when parsing the mid header", header.len));
  }

  if header.mid < 1 || header.mid > 9999 {
    return Err(format!("Invalid mid '{}' when parsing the mid header", header.mid));
  }

  if header.rev > 9999 {
    return Err(format!("Invalid revision '{}' when parsing the mid header", header.rev));
  }

  header.no_ack_flag = field_parse_with_default(MF_NO_ACK_FLAG, raw_mid, 0);
  header.station_id = field_parse_with_default(MF_STATION_ID, raw_mid, 0);
  header.spindle_id = field_parse_with_default(MF_SPINDLE_ID, raw_mid, 0);
  header.sequence_number = field_parse_with_default(MF_SEQUENCE_NUMBER, raw_mid, 0);
  header.number_of_message_parts = field_parse_with_default(MF_NUMBER_OF_MESSAGE_PARTS, raw_mid, 0);
  header.message_part_number = field_parse_with_default(MF_MESSAGE_PART_NUMBER, raw_mid, 0);
  Ok(header)
}

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  IntoStaticStr,
  FromRepr,
  Default,
  EnumString,
  EnumIter,
  AsRefStr
)]
pub enum MidName {
  // Application Communication messages
  #[default]
  #[strum(serialize = "Application Communication start")]
  ApplicationCommunicationStart = 1,
  #[strum(serialize = "Application Communication start acknowledge")]
  ApplicationCommunicationStartAcknowledge = 2,
  #[strum(serialize = "Application Communication stop")]
  ApplicationCommunicationStop = 3,
  #[strum(serialize = "Application Communication negative acknowledge")]
  ApplicationCommunicationNegativeAcknowledge = 4,
  #[strum(serialize = "Application Communication positive acknowledge")]
  ApplicationCommunicationPositiveAcknowledge = 5,
  #[strum(serialize = "Application data message request")]
  ApplicationDataMessageRequest = 6,
  #[strum(serialize = "MID 0007 DOES NOT EXIST")]
  Mid0007DoesNotExist = 7,
  #[strum(serialize = "Application data message subscription")]
  ApplicationDataMessageSubscription = 8,
  #[strum(serialize = "Application Data Message unsubscribe")]
  ApplicationDataMessageUnsubscribe = 9,

  // Application Parameter Set Messages
  #[strum(serialize = "Parameter set ID upload request")]
  ParameterSetIdUploadRequest = 10,
  #[strum(serialize = "Parameter set ID upload reply")]
  ParameterSetIdUploadReply = 11,
  #[strum(serialize = "Parameter set data upload request")]
  ParameterSetDataUploadRequest = 12,
  #[strum(serialize = "Parameter set data upload reply")]
  ParameterSetDataUploadReply = 13,
  #[strum(serialize = "Parameter set selected subscribe")]
  ParameterSetSelectedSubscribe = 14,
  #[strum(serialize = "Parameter set selected")]
  ParameterSetSelected = 15,
  #[strum(serialize = "Parameter set selected acknowledge")]
  ParameterSetSelectedAcknowledge = 16,
  #[strum(serialize = "Parameter set selected unsubscribe")]
  ParameterSetSelectedUnsubscribe = 17,
  #[strum(serialize = "Select Parameter set")]
  SelectParameterSet = 18,
  #[strum(serialize = "Set Parameter set batch size")]
  SetParameterSetBatchSize = 19,
  #[strum(serialize = "Reset Parameter set batch counter")]
  ResetParameterSetBatchCounter = 20,
  #[strum(serialize = "Lock at batch done subscribe")]
  LockAtBatchDoneSubscribe = 21,
  #[strum(serialize = "Lock at batch done upload")]
  LockAtBatchDoneUpload = 22,
  #[strum(serialize = "Lock at batch done upload acknowledge")]
  LockAtBatchDoneUploadAcknowledge = 23,
  #[strum(serialize = "Lock at batch done unsubscribe")]
  LockAtBatchDoneUnsubscribe = 24,
  #[strum(serialize = "Parameter user set download request")]
  ParameterUserSetDownloadRequest = 25,
  #[strum(serialize = "Disable tool")]
  DisableTool = 42,
  #[strum(serialize = "Enable tool")]
  EnableTool = 43,
  #[strum(serialize = "Vehicle ID number download request")]
  VehicleIdNumberDownloadRequest = 50,
  #[strum(serialize = "Vehicle ID number subscribe")]
  VehicleIdNumberSubscribe = 51,
  #[strum(serialize = "Vehicle ID number")]
  VehicleIdNumber = 52,
  #[strum(serialize = "Vehicle ID number acknowledge")]
  VehicleIdNumberAcknowledge = 53,
  #[strum(serialize = "Vehicle ID number unsubscribe")]
  VehicleIdNumberUnsubscribe = 54,
  #[strum(serialize = "Last tightening result data subscribe")]
  LastTighteningResultDataSubscribe = 60,
  #[strum(serialize = "Last tightening result data")]
  LastTighteningResultData = 61,
  #[strum(serialize = "Last tightening result data acknowledge")]
  LastTighteningResultDataAcknowledge = 62,
  #[strum(serialize = "Last tightening result data unsubscribe")]
  LastTighteningResultDataUnsubscribe = 63,
  #[strum(serialize = "Old tightening result upload request")]
  OldTighteningResultUploadRequest = 64,
  #[strum(serialize = "Old tightening result upload reply")]
  OldTighteningResultUploadReply = 65,
  #[strum(serialize = "Number of offline results")]
  NumberOfOfflineResults = 66,
  #[strum(serialize = "Tightening Result List Upload")]
  TighteningResultListUpload = 67,
  #[strum(serialize = "Alarm subscribe")]
  AlarmSubscribe = 70,
  #[strum(serialize = "Alarm")]
  Alarm = 71,
  #[strum(serialize = "Alarm acknowledge")]
  AlarmAcknowledge = 72,
  #[strum(serialize = "Alarm unsubscribe")]
  AlarmUnsubscribe = 73,
  #[strum(serialize = "Read time upload request")]
  ReadTimeUploadRequest = 80,
  #[strum(serialize = "Read time upload reply")]
  ReadTimeUploadReply = 81,
  #[strum(serialize = "Set time")]
  SetTime = 82,
  #[strum(serialize = "Identifier download request")]
  IdentifierDownloadRequest = 150,
  #[strum(serialize = "Set externally controlled relays")]
  SetExternallyControlledRelays = 200,
  #[strum(serialize = "Status externally monitored inputs subscribe")]
  StatusExternallyMonitoredInputsSubscribe = 210,
  #[strum(serialize = "Status externally monitored inputs")]
  StatusExternallyMonitoredInputs = 211,
  #[strum(serialize = "Status externally monitored inputs acknowledge")]
  StatusExternallyMonitoredInputsAcknowledge = 212,
  #[strum(serialize = "Status externally monitored inputs unsubscribe")]
  StatusExternallyMonitoredInputsUnsubscribe = 213,
  #[strum(serialize = "IO device status request")]
  IoDeviceStatusRequest = 214,
  #[strum(serialize = "IO device status reply")]
  IoDeviceStatusReply = 215,
  #[strum(serialize = "Relay function subscribe")]
  RelayFunctionSubscribe = 216,
  #[strum(serialize = "Relay function")]
  RelayFunction = 217,
  #[strum(serialize = "Relay function acknowledge")]
  RelayFunctionAcknowledge = 218,
  #[strum(serialize = "Relay function unsubscribe")]
  RelayFunctionUnsubscribe = 219,
  #[strum(serialize = "Digital input function subscribe")]
  DigitalInputFunctionSubscribe = 220,
  #[strum(serialize = "Digital input function")]
  DigitalInputFunction = 221,
  #[strum(serialize = "Digital input function acknowledge")]
  DigitalInputFunctionAcknowledge = 222,
  #[strum(serialize = "Digital input function unsubscribe")]
  DigitalInputFunctionUnsubscribe = 223,
  #[strum(serialize = "Set digital input function")]
  SetDigitalInputFunction = 224,
  #[strum(serialize = "Reset digital input function")]
  ResetDigitalInputFunction = 225,
  #[strum(serialize = "Selector socket info subscribe")]
  SelectorSocketInfoSubscribe = 250,
  #[strum(serialize = "Selector socket info")]
  SelectorSocketInfo = 251,
  #[strum(serialize = "Selector socket info acknowledge")]
  SelectorSocketInfoAcknowledge = 252,
  #[strum(serialize = "Selector socket info unsubscribe")]
  SelectorSocketInfoUnsubscribe = 253,

  // Application Keep alive message
  #[strum(serialize = "Keep alive message")]
  KeepAliveMessage = 9999,
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn mid_1_name() {
    let mn = MidName::ApplicationCommunicationStart;
    assert_eq!(mn, MidName::from_str("Application Communication start").unwrap());
    assert_eq!(mn, MidName::from_repr(1).unwrap());
  }

  #[test]
  fn mid_2_name() {
    let mn = MidName::ApplicationCommunicationStartAcknowledge;
    assert_eq!(mn, MidName::from_str("Application Communication start acknowledge").unwrap());
    assert_eq!(mn, MidName::from_repr(2).unwrap());
  }

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
    let h = MidHeader::default();
    assert_eq!(h.str(), "00000000000000000000");
  }

  #[test]
  fn mid_header_str_2() {
    let mut h = MidHeader::default();
    h.len = 20;
    assert_eq!(h.str(), "00200000000000000000");
  }

  #[test]
  fn mid_header_str_3() {
    let mut h = MidHeader::default();
    h.len = 20;
    h.mid = 1;
    assert_eq!(h.str(), "00200001000000000000");
  }

  #[test]
  fn mid_header_str_4() {
    let mut h = MidHeader::default();
    h.len = 1234;
    h.mid = 1;
    h.rev = 4;
    h.no_ack_flag = 1;
    assert_eq!(h.str(), "12340001004100000000");
  }

  #[test]
  fn mid_header_str_5() {
    let mut h = MidHeader::default();
    h.len = 1111;
    h.mid = 2222;
    h.rev = 333;
    h.no_ack_flag = 4;
    h.station_id = 55;
    h.spindle_id = 66;
    h.sequence_number = 77;
    h.number_of_message_parts = 8;
    h.message_part_number = 9;
    assert_eq!(h.str(), "11112222333455667789");
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
  fn mid_header_len_20_correct_43_then_spaces() {
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
