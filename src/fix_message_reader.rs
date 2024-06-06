use crate::{
    utils::{FIX_BODY_LEN, FIX_CHECK_SUM, FIX_MESSAGE_TYPE, FIX_VERSION},
    FixMessageBodyBuilder, FixMessageIterator, FixSerializeError,
};

#[derive(Debug)]
pub enum FixMessageReader<'s> {
    AsStr(&'s str),
    AsBytes(&'s [u8]),
}

impl<'s> FixMessageReader<'s> {
    pub fn from_str(data: &'s str) -> Self {
        Self::AsStr(data)
    }

    pub fn from_bytes(data: &'s [u8]) -> Self {
        Self::AsBytes(data)
    }

    pub fn iter(&'s self) -> FixMessageIterator<'s> {
        match &self {
            Self::AsStr(src) => FixMessageIterator::from_str(src),
            Self::AsBytes(src) => FixMessageIterator::from_slice(src),
        }
    }

    pub fn check_payload(self) -> Result<Self, FixSerializeError> {
        let mut fix_version = None;
        let mut fix_body_len = None;
        let mut fix_check_sum = None;

        let mut fix_message_type = None;

        let mut body_builder = FixMessageBodyBuilder::new();

        for itm in self.iter() {
            let itm = itm?;

            match itm.key {
                FIX_VERSION => fix_version = Some(itm),
                FIX_BODY_LEN => fix_body_len = Some(itm),
                FIX_CHECK_SUM => fix_check_sum = Some(itm),
                FIX_MESSAGE_TYPE => {
                    body_builder.append(itm.key, itm.value);
                    fix_message_type = Some(itm);
                }
                _ => body_builder.append(itm.key, itm.value),
            }
        }

        if fix_body_len.is_none() {
            return Err(FixSerializeError::BodyLenTagNotFound);
        }

        //        let fix_body_len = fix_body_len.unwrap();

        if fix_message_type.is_none() {
            return Err(FixSerializeError::MessageTypeTagNotFound);
        }

        //let fix_message_type = fix_message_type.unwrap();

        if fix_check_sum.is_none() {
            return Err(FixSerializeError::CheckSumTagNotFound);
        }

        if fix_version.is_none() {
            return Err(FixSerializeError::VersionTagNotFound);
        }

        let fix_check_sum = fix_check_sum.unwrap();
        let fix_version = fix_version.unwrap();

        let check_sum = body_builder.get_checksum(fix_version.value);

        if check_sum.as_str() != fix_check_sum.value {
            return Err(FixSerializeError::InvalidCheckSum);
        }

        Ok(self)
    }

    pub fn get_value(&self, key: &str) -> Result<Option<&str>, FixSerializeError> {
        for itm in self.iter() {
            let itm = itm?;

            if itm.key == key {
                return Ok(Some(itm.value));
            }
        }

        Ok(None)
    }

    pub fn get_values(&self, key: &str) -> Result<Vec<&str>, FixSerializeError> {
        let mut result = Vec::new();
        for itm in self.iter() {
            let itm = itm?;

            if itm.key == key {
                result.push(itm.value);
            }
        }

        Ok(result)
    }

    pub fn get_message_type(&self) -> Result<&str, FixSerializeError> {
        let value = self.get_value(FIX_MESSAGE_TYPE)?;

        if value.is_none() {
            return Err(FixSerializeError::MessageTypeTagNotFound);
        }

        Ok(value.unwrap())
    }

    pub fn to_string(&self) -> String {
        match self {
            FixMessageReader::AsStr(src) => src.to_string(),
            FixMessageReader::AsBytes(src) => {
                crate::utils::convert_fix_message_to_string(src.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FixMessageReader;

    #[test]
    fn test_check_sum() {
        let fix_string = "8=FIX.4.4|9=75|35=A|34=1092|49=TESTBUY1|52=20180920-18:24:59.643|56=TESTSELL1|98=0|108=60|10=178|";
        FixMessageReader::from_str(fix_string)
            .check_payload()
            .unwrap();
    }

    #[test]
    fn test_check_sum_with_invalid_check_sum() {
        let fix_string = "8=FIX.4.4|9=75|35=A|34=1092|49=TESTBUY1|52=20180920-18:24:59.643|56=TESTSELL1|98=0|108=60|10=179|";
        match FixMessageReader::from_str(fix_string).check_payload() {
            Ok(_) => {
                panic!("Should not be at Ok scenario");
            }
            Err(err) => match err {
                crate::FixSerializeError::InvalidCheckSum => {}
                _ => {
                    panic!("Should not be at scenario: {:?}", err);
                }
            },
        }
    }

    #[test]
    fn test_invalid_fix_no_version() {
        let fix_string =
            b"9=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=178";

        let builder = FixMessageReader::from_bytes(fix_string).check_payload();

        assert!(builder.is_err());
        assert!(builder.err().unwrap().is_version_tag_not_found());
    }

    #[test]
    fn test_invalid_fix_no_message_type() {
        let fix_string =
            b"8=FIX.4.49=75108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=178";
        let builder = FixMessageReader::from_bytes(fix_string).check_payload();

        assert!(builder.is_err());
        assert!(builder.err().unwrap().is_message_type_tag_not_found());
    }

    #[test]
    fn test_no_check_sum_with_validation() {
        let fix_string =
            b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0";
        let builder = FixMessageReader::from_bytes(fix_string).check_payload();

        assert!(builder.is_err());
        assert!(builder.err().unwrap().is_check_sum_tag_not_found());
    }

    #[test]
    fn test_invalid_fix_check_sum() {
        let fix_string = b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=188";
        let builder = FixMessageReader::from_bytes(fix_string).check_payload();

        assert!(builder.is_err());
        assert!(builder.err().unwrap().is_invalid_check_sum());
    }
}
