use crate::openprotocol::core::{MidField, MidHeader, field_parse, mid_parse_header};

// MID 0002

const MF_0002_REV1_CELL_ID: MidField = MidField {
    name: "cell_id",
    rng: 22..26,
};

const MF_0002_REV1_CHANNEL_ID: MidField = MidField {
    name: "channel_id",
    rng: 28..30,
};

const MF_0002_REV1_CONTROLLER_NAME: MidField = MidField {
    name: "controller_name",
    rng: 32..57,
};

const MF_0002_REV2_SUPPLIER_CODE: MidField = MidField {
    name: "supplier_code",
    rng: 59..62,
};

const MF_0002_REV3_OPEN_PROTOCOL_VERSION: MidField = MidField {
    name: "open_protocol_version",
    rng: 64..83,
};

const MF_0002_REV3_CONTROLLER_SOFTWARE_VERSION: MidField = MidField {
    name: "controller_software_version",
    rng: 85..104,
};

const MF_0002_REV3_TOOL_SOFTWARE_VERSION: MidField = MidField {
    name: "tool_software_version",
    rng: 106..125,
};

const MF_0002_REV4_RBU_TYPE: MidField = MidField {
    name: "rbu_type",
    rng: 127..151,
};

const MF_0002_REV4_CONTROLLER_SERIAL_NUMBER: MidField = MidField {
    name: "controller_serial_number",
    rng: 153..163,
};

const MF_0002_REV5_SYSTEM_TYPE: MidField = MidField {
    name: "system_type",
    rng: 165..168,
};

const MF_0002_REV5_SYSTEM_SUB_TYPE: MidField = MidField {
    name: "system_sub_type",
    rng: 170..173,
};

const MF_0002_REV6_STATION_ID: MidField = MidField {
    name: "station_id",
    rng: 181..191,
};

const MF_0002_REV6_STATION_NAME: MidField = MidField {
    name: "station_name",
    rng: 193..218,
};

const MF_0002_REV6_CLIENT_ID: MidField = MidField {
    name: "client_id",
    rng: 220..221,
};

#[derive(Debug, Default)]
pub struct Mid0002 {
    pub header: MidHeader,
    pub rev1: Mid0002Rev1,
    pub rev2: Mid0002Rev2,
    pub rev3: Mid0002Rev3,
    pub rev4: Mid0002Rev4,
    pub rev5: Mid0002Rev5,
    pub rev6: Mid0002Rev6,
}

pub fn mid_parse_0002(data: &str) -> Result<Mid0002, String> {

    let m2 = Mid0002 {
        header: mid_parse_header(data)?,
        rev1: mid_parse_0002_rev1(data)?,
        rev2: mid_parse_0002_rev2(data)?,
        rev3: mid_parse_0002_rev3(data)?,
        rev4: mid_parse_0002_rev4(data)?,
        rev5: mid_parse_0002_rev5(data)?,
        rev6: mid_parse_0002_rev6(data)?,
    };

    if m2.header.mid != 2 {
        return Err(format!(
            "Unexpected mid {} when parsing for mid 2",
            m2.header.mid
        ));
    }

    if m2.header.rev < 1 {
        return Err(format!(
            "Mid revision {} is less the expected rev 1 when parsing mid 2",
            m2.header.mid
        ));
    }

    Ok(m2)
}

/// 5.2.2 MID 0002 Application Communication start acknowledge
/// When accepting the communication start the controller sends as reply, a Communication start
/// acknowledge. This message contains some basic information about the controller, such as cell ID,
/// channel ID, and name.
/// Message sent by: Controller
/// Answer: None
#[derive(Debug, Default)]
pub struct Mid0002Rev1 {
    /// The cell ID is four bytes long specified by four ASCII digits. Range: 0000-9999.
    pub cell_id: String,
    /// The channel ID is two bytes long specified by two ASCII digits. Range: 00-20.
    pub channel_id: String,
    /// The controller name is 25 bytes long and specified by 25 ASCII characters.
    pub controller_name: String,
}

pub fn mid_parse_0002_rev1(data: &str) -> Result<Mid0002Rev1, String> {
    let mut m2r1 = Mid0002Rev1 {
        cell_id: String::new(),
        channel_id: String::new(),
        controller_name: String::new(),
    };

    match field_parse::<String>(MF_0002_REV1_CELL_ID, data) {
        Ok(v) => {
            m2r1.cell_id = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV1_CHANNEL_ID, data) {
        Ok(v) => {
            m2r1.channel_id = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV1_CONTROLLER_NAME, data) {
        Ok(v) => {
            m2r1.controller_name = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    return Ok(m2r1);
}

// MID 0002 REV 002

#[derive(Debug, Default)]
pub struct Mid0002Rev2 {
    /// ACT (supplier code for Atlas Copco Tools) specified by three ASCII characters.
    pub supplier_code: String,
}

pub fn mid_parse_0002_rev2(data: &str) -> Result<Mid0002Rev2, String> {
    let mut m2r2 = Mid0002Rev2 {
        supplier_code: String::new(),
    };

    match field_parse::<String>(MF_0002_REV2_SUPPLIER_CODE, data) {
        Ok(v) => {
            m2r2.supplier_code = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    Ok(m2r2)
}

// MID 0002 REV 003

#[derive(Debug, Default)]
pub struct Mid0002Rev3 {
    /// Open Protocol version. 19 ASCII characters. This
    /// version mirrors the IMPLEMENTED version of the
    /// Open Protocol and is hence not the same as the
    /// version of the specification. This is caused by, for
    /// instance, the possibility of implementation done of
    /// only a subset of the protocol.
    pub open_protocol_version: String,
    /// The controller software version. 19 ASCII characters.
    pub controller_software_version: String,
    /// The tool software version. 19 ASCII characters.
    pub tool_software_version: String,
}

pub fn mid_parse_0002_rev3(data: &str) -> Result<Mid0002Rev3, String> {
    let mut m2r3 = Mid0002Rev3 {
        open_protocol_version: String::new(),
        controller_software_version: String::new(),
        tool_software_version: String::new(),
    };

    match field_parse::<String>(MF_0002_REV3_OPEN_PROTOCOL_VERSION, data) {
        Ok(v) => {
            m2r3.open_protocol_version = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV3_CONTROLLER_SOFTWARE_VERSION, data) {
        Ok(v) => {
            m2r3.controller_software_version = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV3_TOOL_SOFTWARE_VERSION, data) {
        Ok(v) => {
            m2r3.tool_software_version = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    Ok(m2r3)
}

// MID 0002 REV 004

#[derive(Debug, Default)]
pub struct Mid0002Rev4 {
    /// The RBU Type. 24 ASCII characters.
    pub rbu_type: String,
    /// The Controller Serial Number. 10 ASCII characters.
    pub controller_serial_number: String,
}

pub fn mid_parse_0002_rev4(data: &str) -> Result<Mid0002Rev4, String> {
    let mut m2r4 = Mid0002Rev4 {
        rbu_type: String::new(),
        controller_serial_number: String::new(),
    };

    match field_parse::<String>(MF_0002_REV4_RBU_TYPE, data) {
        Ok(v) => {
            m2r4.rbu_type = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV4_CONTROLLER_SERIAL_NUMBER, data) {
        Ok(v) => {
            m2r4.controller_serial_number = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    Ok(m2r4)
}

// MID 0002 REV 005

#[derive(Debug, Default)]
pub struct Mid0002Rev5 {
    /// The system type of the controller. 3 ASCII digits
    /// Possible values are:
    /// 000 = System type not set
    /// 001 = Power Focus 4000
    /// 002 = Power MACS 4000
    /// 003 = Power Focus 6000
    /// 004 = Micro Torque Focus 6000
    pub system_type: String,
    /// The system subtype. 3 ASCII digits
    /// If no subtype exists it will be set to 000
    /// For a Power Focus 4000 and PF 6000 system the valid subtypes are:
    /// 001 = a normal tightening system
    /// For a Power MACS 4000 system the valid subtypes are:
    /// 001 = a normal tightening system
    /// 002 = a system running presses instead of spindles.
    pub system_sub_type: String,
}

pub fn mid_parse_0002_rev5(data: &str) -> Result<Mid0002Rev5, String> {
    let mut m2r5 = Mid0002Rev5 {
        system_type: String::new(),
        system_sub_type: String::new(),
    };

    match field_parse::<String>(MF_0002_REV5_SYSTEM_TYPE, data) {
        Ok(v) => {
            m2r5.system_type = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV5_SYSTEM_SUB_TYPE, data) {
        Ok(v) => {
            m2r5.system_sub_type = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    Ok(m2r5)
}

// MID 0002 REV 006
#[derive(Debug, Default)]
pub struct Mid0002Rev6 {
    /// The station id/Cell Id is a unique id for each station.
    /// 10 ASCII digits. Max 4294967295
    pub station_id: String,
    /// The station/Cell name is 25 bytes long and specified
    /// by 25 ASCII characters.
    pub station_name: String,
    /// The Connection Client ID.1 byte 1 ASCII digit. Used
    /// at several connections towards a one channel
    /// controller.
    pub client_id: String,
}

pub fn mid_parse_0002_rev6(data: &str) -> Result<Mid0002Rev6, String> {
    let mut m2r6 = Mid0002Rev6 {
        station_id: String::new(),
        station_name: String::new(),
        client_id: String::new(),
    };

    match field_parse::<String>(MF_0002_REV6_STATION_ID, data) {
        Ok(v) => {
            m2r6.station_id = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV6_STATION_NAME, data) {
        Ok(v) => {
            m2r6.station_name = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    match field_parse::<String>(MF_0002_REV6_CLIENT_ID, data) {
        Ok(v) => {
            m2r6.client_id = v;
        }
        Err(e) => return Err(format!("{}", e)),
    }

    Ok(m2r6)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn mid_0002_parse_valid_mid_0002_rev_1_1() {
        let m2 = mid_parse_0002("00200002001000000000as;ldfkja;lskdjf;laskjdf;laskjdf;ljaksd;lfja;lsdjkf;alskjdf;lakjsdf;lajsdf;lajd;fla").unwrap();
        println!("{:#?}", m2);
        assert!(m2.header.mid == 2);
        assert!(m2.header.rev == 1);
        assert!(m2.header.len == 20);
    }

    #[test]
    fn mid2_rev2_valid() {
        let _ = mid_parse_0002_rev2("0123000200200000asfadjfhe0uhiudlkjsdfalsdfjldfjajfdjflksdhgsdfg2143123412341234safdasdfasdf").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid2_rev2_invalid_short_1() {
        let _ = mid_parse_0002_rev2("01230002002").unwrap();
    }

    #[test]
    fn mid2_rev3_valid() {
        let _ = mid_parse_0002_rev3("012300020030000000000000000asfadjfhe0uhiudlkjkjhkljhlkjhgggjhgfjhfhjgfjhfsdfalsdfjldfjajfdjflksdhgsdfg2143123412341234safdasdfasdf").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid2_rev3_invalid_short_1() {
        let _ = mid_parse_0002_rev3("01230002003").unwrap();
    }

    #[test]
    fn mid2_rev4_valid() {
        let _ = mid_parse_0002_rev4("01230002004000000000000000asfadjfhe0uhiudlkjsdfalsdfjhlkhkljhlkjhlkjhoiyoiuyoiuyioyutrtyrtetrerwfgfdsgfxcvxbvxgdfldflkjhkljhkljhlkjhlkjhlkjhlkjhlkjhlkjhlkhkljjajfdjflksdhgsdfg2143123412341234safdasdfasdf").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid2_rev4_invalid_short_1() {
        let _ = mid_parse_0002_rev4("01230002004").unwrap();
    }

    #[test]
    fn mid2_rev5_valid() {
        let _ = mid_parse_0002_rev5("01230002005000000000000000asfadjfhe0uhiudlkjsdfalsdfjhlkhkljhlkjhlkjhoiyoiuyoiuyioyutrtyrtetrerwfgfdsgfxcvxbvxgdfldflkjhkljhkljhlkjhlkjhlkjhlkjhlkjhlkjhlkhkljjajfdjflksdhgsdfg2143123412341234safdasdfasdf").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid2_rev5_invalid_short_1() {
        let _ = mid_parse_0002_rev5("01230002005").unwrap();
    }

    #[test]
    fn mid2_rev6_valid() {
        let _ = mid_parse_0002_rev6("01230002006000000000000000asfadjfhe0uhibnmbnmbnmbnmbnmbnmudlkjsdfalsdfjhlkhkljhlkjhlkjhoiyoiuyoiuyibnmghjfgvbnbnnvnbvnbvnvnboyutrtyrtetrerwfgfdsgfxcvxbvxgdfldflkjhkljhkljhlkjhlkjhlkjhlkjhlkjhlkjhlkhkljjajfdjflksdhgsdfg2143123412341234safdasdfasdf").unwrap();
    }

    #[test]
    #[should_panic]
    fn mid2_rev6_invalid_short_1() {
        let _ = mid_parse_0002_rev6("01230002006").unwrap();
    }
}
