use std::convert::TryFrom;
use std::io;

pub type PiecewiseVersion = (u8, u8, u8, u8);

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Version {
    Unicode1_0(PiecewiseVersion),
    Unicode1_0_1(PiecewiseVersion),
    Unicode1_1_0(PiecewiseVersion),
    Unicode1_1_5(PiecewiseVersion),
    Unicode2_0(PiecewiseVersion),
    Unicode2_1_2(PiecewiseVersion),
    Unicode2_1_5(PiecewiseVersion),
    Unicode2_1_8(PiecewiseVersion),
    Unicode2_1_9(PiecewiseVersion),
    Unicode3_0(PiecewiseVersion),
    Unicode3_0_1(PiecewiseVersion),
    Unicode3_1_0(PiecewiseVersion),
    Unicode3_1_1(PiecewiseVersion),
    Unicode3_2(PiecewiseVersion),
    Unicode4_0(PiecewiseVersion),
    Unicode4_0_1(PiecewiseVersion),
    Unicode4_1(PiecewiseVersion),
    Unicode5_0(PiecewiseVersion),
    Unicode5_1(PiecewiseVersion),
    Unicode5_2(PiecewiseVersion),
    Unicode6_0(PiecewiseVersion),
    Unicode6_1(PiecewiseVersion),
    Unicode6_2(PiecewiseVersion),
    Unicode6_3(PiecewiseVersion),
    Unicode7_0(PiecewiseVersion),
    Unicode8_0(PiecewiseVersion),
    Unicode9_0(PiecewiseVersion),
    Unicode10_0(PiecewiseVersion),
}

// TODO make this more than a stub
impl TryFrom<PiecewiseVersion> for Version {
    type Error = io::Error;

    fn try_from(version: PiecewiseVersion) -> Result<Self, Self::Error> {
        Ok(Version::Unicode10_0(version))
    }
}
