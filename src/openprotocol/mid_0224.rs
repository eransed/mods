use crate::openprotocol::{
  core::{Mid, MidField, MidHeader, field_parse, mid_parse_header},
  digital_input_function::DigitalInputFunction,
};
const MF_FUNCTION: MidField = MidField { name: "digital_input_function", rng: 20..23 };
#[derive(Debug, Clone, Copy)]
pub struct Mid0224 {
  pub header: MidHeader,
  pub digital_input_function: DigitalInputFunction,
}
impl Mid for Mid0224 {
  fn str(&self) -> String {
    let mut header = self.header;
    header.len = 23;
    format!("{}{:03}", header.str(), self.digital_input_function.value())
  }
}
pub fn mid_parse_0224(data: &str) -> Result<Mid0224, String> {
  let header = mid_parse_header(data)?;
  if header.mid != 224 {
    return Err(format!("Unexpected mid {} when parsing for mid 224", header.mid));
  }
  let value = field_parse::<u16>(MF_FUNCTION, data)?;
  Ok(Mid0224 { header, digital_input_function: DigitalInputFunction::try_from(value)? })
}
