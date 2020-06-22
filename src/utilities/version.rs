
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct Version {
    pub major: u8,
    pub minor: u8
}

impl Version {
    pub fn new(major: u8, minor: u8) -> Self {
        Version {
            major,
            minor
        }
    }
}