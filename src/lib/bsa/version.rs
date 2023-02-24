#[derive(Default, PartialEq)]
#[repr(u32)]
pub enum Version {
    V103 = 103,
    V104 = 104,
    #[default]
    V105 = 105,
}

impl From<&Version> for u32 {
    fn from(ver: &Version) -> u32 {
        match ver {
            Version::V103 => 103,
            Version::V104 => 104,
            Version::V105 => 105,
        }
    }
}

impl TryFrom<u32> for Version {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            103 => Ok(Version::V103),
            104 => Ok(Version::V104),
            105 => Ok(Version::V105),
            _ => Err(format!("invalid version {value}")),
        }
    }
}

impl TryFrom<Option<&String>> for Version {
    type Error = String;

    fn try_from(value: Option<&String>) -> Result<Self, Self::Error> {
        match value {
            Some(s) => match s.as_str() {
                "103" => Ok(Version::V103),
                "104" => Ok(Version::V104),
                "105" => Ok(Version::V105),
                _ => Err(format!("invalid version {s}")),
            },
            None => Ok(Version::default()),
        }
    }
}

impl From<&Version> for String {
    fn from(ver: &Version) -> Self {
        match ver {
            Version::V103 => String::from("103"),
            Version::V104 => String::from("104"),
            Version::V105 => String::from("105"),
        }
    }
}
