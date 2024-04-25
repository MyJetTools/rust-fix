#[derive(Debug)]
pub enum FixSerializeError {
    VersionTagNotFound,
    MessageTypeTagNotFound,
    CheckSumTagNotFound,
    InvalidCheckSum,
    FixDelimiterNotFound,
    BodyLenTagNotFound,
}

impl FixSerializeError {
    pub fn is_version_tag_not_found(&self) -> bool {
        println!("{:?}", self);
        match self {
            Self::VersionTagNotFound => true,
            _ => false,
        }
    }

    pub fn is_message_type_tag_not_found(&self) -> bool {
        match self {
            Self::MessageTypeTagNotFound => true,
            _ => false,
        }
    }

    pub fn is_check_sum_tag_not_found(&self) -> bool {
        match self {
            Self::CheckSumTagNotFound => true,
            _ => false,
        }
    }

    pub fn is_invalid_check_sum(&self) -> bool {
        match self {
            Self::InvalidCheckSum => true,
            _ => false,
        }
    }
}
