use super::bindings::*;

#[derive(Clone, Copy, Debug)]
pub struct SenselError;

pub fn sensel_result(status: SenselStatus) -> Result<(), SenselError> {
    match status {
        SenselStatus::SENSEL_OK => Ok(()),
        _ => Err(SenselError)
    }
}
