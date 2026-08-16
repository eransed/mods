use crate::openprotocol::core::{MidField, MidHeader};

/// 5.2.1 MID 0001 Application Communication start
/// This message enables the communication. The controller does not respond to any other command
/// before this
/// Message sent by: Integrator
/// Answers: MID 0002 Communication start acknowledge or
/// MID 0004 Command error, Client already connected or MID revision unsupported
/// Example: Communication start with call for MID 0002 Communication start acknowledge revision 3.
/// 00200001003 NUL
#[derive(Debug)]
pub struct Mid0001Rev7 {
    pub header: MidHeader,

    /// Telling the Open Protocol server that keep alive messages shall be used or not.
    /// 0=Use Keep alive (Keep alive is mandatory) 1=Ignore Keep alive (keep alive is optional)
    pub optional_keep_alive: bool,
}

const MF_0001_REV7_OPTIONAL_KEEP_ALIVE: MidField = MidField {
    name: "optional_keep_alive",
    rng: 22..23,
};
