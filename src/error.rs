use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SRIDError {
    got: Option<u32>,
    want: u32,
}

impl SRIDError {
    pub fn new(got: Option<u32>, want: u32) -> Self {
        Self { got, want }
    }
}

pub fn check_srid(got: Option<u32>, want: u32) -> Result<(), SRIDError> {
    if got != Some(want) {
        Err(SRIDError::new(got, want))
    } else {
        Ok(())
    }
}

impl fmt::Display for SRIDError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wrong SRID in database: {:?}, Expected: {:?}", self.got, self.want)
    }
}

impl std::error::Error for SRIDError {}
