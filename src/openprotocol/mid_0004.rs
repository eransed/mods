use num_derive::FromPrimitive;
use crate::openprotocol::core::Mid;
use crate::openprotocol::core::MidField;
use crate::openprotocol::core::MidHeader;
use crate::openprotocol::core::field_parse;
use crate::openprotocol::core::mid_parse_header;

const MF_0004_MID_NUMBER: MidField = MidField {
    name: "mid_number",
    rng: 20..24,
};

const MF_0004_ERROR_CODE: MidField = MidField {
    name: "error_code",
    rng: 24..26,
};
/// 5.2.4 MID 0004 Application Communication negative acknowledge
/// 
/// This message is used by the controller when a request, command or subscription for any reason has
/// not been performed. The data field contains the message ID of the message request that failed as well
/// as an error code.
/// 
/// It can also be used by the integrator to acknowledge received subscribed data/events then do all the special subscription data acknowledges obsolete.
/// upload and will
/// 
/// When using the communication acknowledgement of MID 0007 and MID 0006 together with
/// sequence numbering this is an application level message only.
/// 
/// For detailed description of use of this message, please look at each Request, Subscription or Command
/// 
/// MIDs description.
/// 
/// Message sent by: Controller:
/// 
/// Answer: None
#[derive(Debug, Copy, Clone, Default)]
pub struct Mid0004 {
    pub header: MidHeader,
    /// MID number (rejected mid number)
    pub mid_number: u16,
    /// Error code for the sent message
    pub error_code: Mid0004ErrorCode,
}

impl Mid for Mid0004 {
    fn str(&self) -> String {
        let mut s = self.header.str();
        s.push_str(format!("{:04}", self.mid_number).as_str());
        s.push_str(format!("{:02}", self.error_code as i32).as_str());
        return s;
    }
}

pub fn mid_parse_0004(data: &str) -> Result<Mid0004, String> {
    let mut m4 = Mid0004 {
        header: mid_parse_header(data)?,
        mid_number: 0,
        error_code: Mid0004ErrorCode::NoError,
    };

    if m4.header.mid != 4 {
        return Err(format!(
            "Unexpected mid {} when parsing for mid 4",
            m4.header.mid
        ));
    }

    match field_parse::<u16>(MF_0004_MID_NUMBER, data) {
        Ok(v) => {
            m4.mid_number = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<i32>(MF_0004_ERROR_CODE, data) {
        Ok(v) => {
            match num::FromPrimitive::from_i32(v) {
                Some(ec) => m4.error_code = ec,
                None => {
                    return Err(format!("Unvalid error code {}", v));
                }
            };
        }
        Err(e) => return Err(format!("{}", e)),
    }
    return Ok(m4);
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

use crate::openprotocol::core::MidName;

use super::*;

  #[test]
    fn mid_0004_name() {
        assert_eq!(MidName::from_repr(4).unwrap(), MidName::from_str("Application Communication negative acknowledge").unwrap());
    }

    #[test]
    fn mid_0004_parse_valid_mid_0004_rev_1_1() {
        let m4 = mid_parse_0004("00260004001000000000123499").unwrap();
        assert!(m4.header.mid == 4);
        assert!(m4.header.rev == 1);
        assert!(m4.header.len == 26);
        assert!(m4.mid_number == 1234);
        assert!(m4.error_code == Mid0004ErrorCode::UnknownMID);
    }

    #[test]
    fn mid_0004_parse_valid_mid_0004_rev_1_2() {
        let _ = mid_parse_0004("00260004001000000000020001").unwrap();
    }

    #[test]
    fn mid_0004_parse_valid_mid_0004_rev_1_3() {
        let _ = mid_parse_0004("00260004001000000000432199").unwrap();
    }

    #[test]
    fn mid_0004_parse_valid_mid_0004_rev_1_4() {
        let _ = mid_parse_0004("00260004001000000000234579").unwrap();
    }

    // Expected errors
    #[test]
    #[should_panic]
    fn mid_0004_parse_valid_mid_0005_rev_1_should_error_1() {
        let _ = mid_parse_0004("0022000500100000000099").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_mid_with_invalid_len_1() {
        let _ = mid_parse_0004("222000500100000000099").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_mid_with_invalid_len_2() {
        let _ = mid_parse_0004("0000000500100000000099").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_mid_with_invalid_len_3() {
        let _ = mid_parse_0004("aaaabbbbccc0000000001100").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_mid_with_invalid_len_4() {
        let _ = mid_parse_0004("1234ddddeee0000000001100").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_invalid_error_code_mid_0004_rev_1_1() {
        let _ = mid_parse_0004("00220004001000000000").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_one_char_too_short_mid_0004_rev_1() {
        let _ = mid_parse_0004("002200040010000000009").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_offset_left_by_one_mid_0004_rev_1() {
        let _ = mid_parse_0004("022000400100000000099").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_only_zeros_mid_0004_rev_1() {
        let _ = mid_parse_0004("0000000000000000000000").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_only_ones_mid_0004_rev_1() {
        let _ = mid_parse_0004("1111111111111111111111").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_only_blankspace_mid_0004_rev_1() {
        let _ = mid_parse_0004("                      ").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_only_blankspace_correct_len_mid_0004_rev_1() {
        let _ = mid_parse_0004("0022                  ").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid_0004_parse_only_random_numbers_mid_0004_rev_1() {
        let _ = mid_parse_0004("0022474528364523840873").unwrap();
    }
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq, Default)]
pub enum Mid0004ErrorCode {
    #[default] NoError = 0,
    InvalidData = 1,
    ParmeterSetIdNotPresent = 2,
    ParameterSetCanNotBeSet = 3,
    ParameterSetNotRunning = 4,
    VINUploadSubscriptionAlreadyExists = 6,
    VINUploadSubscriptionDoesNotExists = 7,
    VINInputSourceNotGranted = 8,
    LastTighteningResultSubscriptionAlreadyExists = 9,
    LastTighteningResultSubscriptionDoesNotExist = 10,
    AlarmSubscriptionAlreadyExists = 11,
    AlarmSubscriptionDoesNotExist = 12,
    ParameterSetSelectionSubscriptionAlreadyExists = 13,
    ParameterSetSelectionSubscriptionSoesNotExist = 14,
    TighteningIDRequestedNotFound = 15,
    ConnectionRejectedProtocolBusy = 16,
    JobIDNotPresent = 17,
    JobInfoSubscriptionAlreadyExists = 18,
    JobInfoSubscriptionDoesNotExist = 19,
    JobCanNotBeSet = 20,
    JobNotRunning = 21,
    NotPossibleToExecuteDynamicJobRequest = 22,
    JobBatchDecrementFailed = 23,
    NotPossibleToCreatePset = 24,
    ProgrammingControlNotGranted = 25,
    WrongToolTypeToPsetDownloadConnected = 26,
    ToolIsInaccessible = 27,
    JobAbortionIsInProgress = 28,
    ToolDoesNotExist = 29,
    ControllerIsNotASyncMasterStationController = 30,
    MultiSpindleStatusSubscriptionAlreadyExists = 31,
    MultiSpindleStatusSubscriptionDoesNotExist = 32,
    MultiSpindleResultSubscriptionAlreadyExists = 33,
    MultiSpindleResultSubscriptionDoesNotExist = 34,
    OtherMasterClientAlreadyConnected = 35,
    LockTypeNotSupported = 36,
    JobLineControlInfoSubscriptionAlreadyExists = 40,
    JobLineControlInfoSubscriptionDoesNotExist = 41,
    IdentifierInputSourceNotGranted = 42,
    MultipleIdentifiersWorkOrderSubscriptionAlreadyExists = 43,
    MultipleIdentifiersWorkOrderSubscriptionDoesNotExist = 44,
    StatusExternalMonitoredInputsSubscriptionAlreadyExists = 50,
    StatusExternalMonitoredInputsSubscriptionDoesNotExists = 51,
    IODeviceNotConnected = 52,
    FaultyIODeviceID = 53,
    ToolTagIDUnknown = 54,
    ToolTagIDSubscriptionAlreadyExists = 55,
    ToolTagIDSubscriptionDoesNotExists = 56,
    ToolMotorTuningFailed = 57,
    NoAlarmPresent = 58,
    ToolCurrentlyInUse = 59,
    NoHistogramAvailable = 60,
    PairingFailed = 61,
    PairingDenied = 62,
    PairingOrPairingAbortionAttemptOnWrongTooltype = 63,
    PairingAbortionDenied = 64,
    PairingAbortionFailed = 65,
    PairingDisconnectionFailed = 66,
    PairingInProgressOrAlreadyDone = 67,
    PairingDeniedNoProgramControl = 68,
    UnsupportedExtraDataRevision = 69,
    CalibrationFailed = 70,
    SubscriptionAlreadyExists = 71,
    SubscriptionDoesNotExists = 72,
    SubscribedMIDUnsupported = 73,
    SubscribedMIDRevisionUnsupported = 74,
    RequestedMIDUnsupported = 75,
    RequestedMIDRevisionUnsupported = 76,
    RequestedOnSpecificDataNotSupported = 77,
    SubscriptionOnSpecificDataNotSupported = 78,
    CommandFailed = 79,
    AudiEmergencyStatusSubscriptionExists = 80,
    AudiEmergencyStatusSubscriptionDoesNotExists = 81,
    AutomaticManualModeSubscribeAlreadyExist = 82,
    AutomaticManualModeSubscribeDoesNotExist = 83,
    TheRelayFunctionSubscriptionAlreadyExists = 84,
    TheRelayFunctionSubscriptionDoesNotExist = 85,
    TheSelectorSocketInfoSubscriptionAlreadyExist = 86,
    TheSelectorSocketInfoSubscriptionDoesNotExist = 87,
    TheDiginInfoSubscriptionAlreadyExist = 88,
    TheDiginInfoSubscriptionDoesNotExist = 89,
    LockAtBatchDoneSubscriptionAlreadyExist = 90,
    LockAtBatchDoneSubscriptionDoesNotExist = 91,
    OpenProtocolCommandsDisabled = 92,
    OpenProtocolCommandsDisabledSubscriptionAlreadyExists = 93,
    OpenProtocolCommandsDisabledSubscriptionDoesNotExist = 94,
    RejectRequestPowerMACSIsInManualMode = 95,
    RejectConnectionClientAlreadyConnected = 96,
    MIDRevisionUnsupported = 97,
    ControllerInternalRequestTimeout = 98,
    UnknownMID = 99,
}
