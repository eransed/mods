use crate::openprotocol::core::{MidField, MidHeader, mid_header_str};

/// 5.2.1 MID 0001 Application Communication start
/// 
/// This message enables the communication. The controller does not respond to any other command
/// before this
/// 
/// Message sent by: Integrator
/// 
/// Answers: MID 0002 Communication start acknowledge or
/// 
/// MID 0004 Command error, Client already connected or MID revision unsupported
/// 
/// Example: Communication start with call for MID 0002 Communication start acknowledge revision 3.
/// 
/// 00200001003 NUL

#[derive(Debug, Default)]
pub struct Mid0001 {
    pub header: MidHeader,
    pub rev7: Mid0001Rev7,
}

const MF_0001_REV7_OPTIONAL_KEEP_ALIVE: MidField = MidField {
    name: "optional_keep_alive",
    rng: 22..23,
};

#[derive(Debug, Default)]
pub struct Mid0001Rev7 {
    /// Telling the Open Protocol server that keep alive messages shall be used or not.
    /// 0=Use Keep alive (Keep alive is mandatory) 1=Ignore Keep alive (keep alive is optional)
    pub optional_keep_alive: bool,
}

pub fn mid_parse_mid_0001(data: &str) -> Result<Mid0001, String> {
    let mut m1 = Mid0001 {
        header: crate::openprotocol::core::mid_parse_header(data)?,
        rev7: Mid0001Rev7 {
            optional_keep_alive: false,
        },
    };

    if m1.header.mid != 1 {
        return Err(format!(
            "Unexpected mid {} when parsing for mid 1",
            m1.header.mid
        ));
    }

    if m1.header.rev == 7 {
        let optional_keep_alive = &data[MF_0001_REV7_OPTIONAL_KEEP_ALIVE.rng.clone()];
        m1.rev7.optional_keep_alive = optional_keep_alive == "0";
    }

    Ok(m1)
}

pub fn mid_0001_to_string(m: Mid0001) -> String {
    if m.header.rev == 7 {
        let mut s = mid_header_str(m.header);
        s.push_str(format!("01{}", if m.rev7.optional_keep_alive { "0" } else { "1" }).as_str());
        return s;
    }
    return mid_header_str(m.header);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn mid_0001_parse_valid_mid_0001() {
        let m1 = mid_parse_mid_0001("00200001001000000000").unwrap();
        assert_eq!(m1.header.mid, 1);
        assert_eq!(m1.header.rev, 1);
        assert_eq!(m1.header.len, 20);
    }

    #[test]
    fn mid_0001_parse_valid_mid_0001_rev7() {
        let m1 = mid_parse_mid_0001("00230001007000000000010").unwrap();
        assert_eq!(m1.header.mid, 1);
        assert_eq!(m1.header.rev, 7);
        assert_eq!(m1.header.len, 23);
        assert_eq!(m1.rev7.optional_keep_alive, true);
    }

    #[test]
    fn mid_0001_serialize() {
        let mut m1 = Mid0001 {
            header: MidHeader::default(),
            rev7: Mid0001Rev7 {
                optional_keep_alive: false,
            },
        };
        m1.header.mid = 1;
        m1.header.rev = 7;
        m1.header.len = 23;
        let s = mid_0001_to_string(m1);
        assert_eq!(s, "00230001007000000000011");
    }
}

