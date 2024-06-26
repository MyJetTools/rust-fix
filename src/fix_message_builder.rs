use crate::{
    utils::{bytes_to_fix_string, calculate_check_sum, compile_fix_chunk, split_fix_to_tags},
    FixSerializeError,
};

#[derive(Clone)]
pub struct FixMessageBuilder {
    fix_version: Vec<u8>,
    message_type: Vec<u8>,
    data: Vec<(Vec<u8>, Vec<u8>)>,
}

impl FixMessageBuilder {
    pub fn from_bytes(
        payload: &[u8],
        check_sum_validation: bool,
    ) -> Result<Self, FixSerializeError> {
        let tags = split_fix_to_tags(payload);

        let Some(version) = tags.get(crate::utils::FIX_VERSION.as_bytes()) else {
            println!(
                "Tag not found: {:?}. Str: {}",
                payload,
                std::str::from_utf8(payload).unwrap()
            );

            return Err(FixSerializeError::VersionTagNotFound);
        };

        let Some(message_type) = tags.get(crate::utils::FIX_MESSAGE_TYPE.as_bytes()) else {
            return Err(FixSerializeError::MessageTypeTagNotFound);
        };

        let source_check_sum = tags.get(crate::utils::FIX_CHECK_SUM.as_bytes());

        if check_sum_validation == true && source_check_sum.is_none() {
            return Err(FixSerializeError::CheckSumTagNotFound);
        }

        let version = version.first().unwrap();
        let message_type = message_type.first().unwrap();

        let mut result = Self {
            fix_version: version.clone(),
            message_type: message_type.clone(),
            data: vec![],
        };

        let to_skip = vec![
            crate::utils::FIX_BODY_LEN.as_bytes(),
            crate::utils::FIX_VERSION.as_bytes(),
            crate::utils::FIX_CHECK_SUM.as_bytes(),
        ];

        for (tag, values) in &tags {
            for value in values {
                if to_skip.contains(&tag.as_slice()) {
                    continue;
                }

                result.with_value_as_bytes(tag.clone(), value.clone())
            }
        }

        if check_sum_validation {
            if source_check_sum.unwrap().first().unwrap()
                != &result.calculate_check_sum().as_bytes().to_vec()
            {
                return Err(FixSerializeError::InvalidCheckSum);
            }
        }

        return Ok(result);
    }

    pub fn new(version: &str, message_type: &str) -> Self {
        return Self {
            fix_version: version.as_bytes().to_vec(),
            message_type: message_type.as_bytes().to_vec(),
            data: vec![],
        };
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        return self.compile_message();
    }

    pub fn get_value(&self, key: Vec<u8>) -> Option<&Vec<u8>> {
        for (inner_key, value) in &self.data {
            if inner_key == &key {
                return Some(value);
            }
        }

        return None;
    }

    pub fn get_values(&self, key: Vec<u8>) -> Vec<&Vec<u8>> {
        let mut result = vec![];

        for (inner_key, value) in &self.data {
            if inner_key == &key {
                result.push(value)
            }
        }

        return result;
    }

    pub fn get_message_type(&self) -> &Vec<u8> {
        return &self.message_type;
    }

    pub fn get_message_type_as_string(&self) -> String {
        return String::from_utf8(self.message_type.clone()).unwrap();
    }

    pub fn get_value_as_string(&self, key: Vec<u8>) -> Option<String> {
        for (inner_key, value) in &self.data {
            if inner_key == &key {
                return Some(String::from_utf8(value.clone()).unwrap());
            }
        }

        return None;
    }

    pub fn get_values_as_string(&self, key: Vec<u8>) -> Vec<String> {
        let mut result = vec![];
        for (inner_key, value) in &self.data {
            if inner_key == &key {
                result.push(String::from_utf8(value.clone()).unwrap());
            }
        }

        return result;
    }

    pub fn get_value_string(&self, key: &str) -> Option<String> {
        for (inner_key, value) in &self.data {
            if inner_key == &key.as_bytes() {
                return Some(String::from_utf8(value.clone()).unwrap());
            }
        }

        return None;
    }

    pub fn get_values_string(&self, key: &str) -> Vec<String> {
        let mut result = vec![];
        for (inner_key, value) in &self.data {
            if inner_key == &key.as_bytes() {
                result.push(String::from_utf8(value.clone()).unwrap());
            }
        }

        return result;
    }

    pub fn with_value(&mut self, key: i32, value: &str) {
        self.data.push((
            key.to_string().as_bytes().to_vec(),
            value.as_bytes().to_vec(),
        ));
    }

    fn with_value_as_bytes(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.push((key, value));
    }

    fn compile_message(&self) -> Vec<u8> {
        let mut result = compile_fix_chunk(crate::utils::FIX_VERSION.as_bytes(), &self.fix_version);

        let (body_len, body) = self.compile_body();

        result.extend_from_slice(&compile_fix_chunk(
            crate::utils::FIX_BODY_LEN.as_bytes(),
            body_len.to_string().as_bytes(),
        ));
        result.extend_from_slice(&body);

        result.extend_from_slice(&compile_fix_chunk(
            crate::utils::FIX_CHECK_SUM.as_bytes(),
            calculate_check_sum(&result).as_bytes(),
        ));

        return result;
    }

    fn calculate_check_sum(&self) -> String {
        let mut result = compile_fix_chunk(crate::utils::FIX_VERSION.as_bytes(), &self.fix_version);

        let (body_len, body) = self.compile_body();

        result.extend_from_slice(&compile_fix_chunk(
            crate::utils::FIX_BODY_LEN.as_bytes(),
            body_len.to_string().as_bytes(),
        ));
        result.extend_from_slice(&body);

        return calculate_check_sum(&result);
    }

    fn compile_body(&self) -> (usize, Vec<u8>) {
        let mut body: Vec<u8> = compile_fix_chunk(
            crate::utils::FIX_MESSAGE_TYPE.as_bytes(),
            &self.message_type,
        );

        for (key, value) in &self.data {
            let data_to_insert = compile_fix_chunk(key, value);
            body.extend_from_slice(&data_to_insert)
        }

        return (body.len(), body);
    }
}

impl ToString for FixMessageBuilder {
    fn to_string(&self) -> String {
        let bytes = self.compile_message();
        return bytes_to_fix_string(&bytes);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_fix_string() {
        let fix_string = "8=FIX.4.4|9=75|35=A|34=1092|49=TESTBUY1|52=20180920-18:24:59.643|56=TESTSELL1|98=0|108=60|10=178|";

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(34, &"1092".to_string());
        fix_builder.with_value(49, &"TESTBUY1".to_string());
        fix_builder.with_value(52, &"20180920-18:24:59.643".to_string());
        fix_builder.with_value(56, &"TESTSELL1".to_string());
        fix_builder.with_value(98, &"0".to_string());
        fix_builder.with_value(108, &"60".to_string());

        let fix_to_assert: String = fix_builder.to_string();

        assert_eq!(fix_string, &fix_to_assert);
    }

    #[test]
    fn test_invalid_fix_no_version() {
        let fix_string =
            b"9=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=178";

        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::VersionTagNotFound as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_invalid_fix_no_message_type() {
        let fix_string =
            b"8=FIX.4.49=75108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=178";
        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::MessageTypeTagNotFound as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_no_check_sum_with_validation() {
        let fix_string =
            b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0";
        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::CheckSumTagNotFound as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_no_check_sum_with_disabled_validation() {
        let fix_string =
            b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0";
        let builder = FixMessageBuilder::from_bytes(fix_string, false);

        assert_eq!(false, builder.is_err());
    }
    #[test]
    fn test_invalid_fix_check_sum_with_disabled_validation() {
        let fix_string = b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=188";
        let builder = FixMessageBuilder::from_bytes(fix_string, false);

        assert_eq!(false, builder.is_err());
    }

    #[test]
    fn test_invalid_fix_check_sum() {
        let fix_string = b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=188";
        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::InvalidCheckSum as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_to_bytes() {
        let fix_string = b"8=FIX.4.49=7535=A34=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0108=6010=178";

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(34, &"1092".to_string());
        fix_builder.with_value(49, &"TESTBUY1".to_string());
        fix_builder.with_value(52, &"20180920-18:24:59.643".to_string());
        fix_builder.with_value(56, &"TESTSELL1".to_string());
        fix_builder.with_value(98, &"0".to_string());
        fix_builder.with_value(108, &"60".to_string());

        let fix_to_assert = fix_builder.as_bytes();

        assert_eq!(fix_string, fix_to_assert.as_slice());
    }

    #[test]
    fn test_few_values_with_same_tag() {
        let fix_string = b"8=FIX.4.49=8735=A34=109249=TESTBUY149=TESTBUY252=20180920-18:24:59.64356=TESTSELL198=0108=6010=194";

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(34, &"1092".to_string());
        fix_builder.with_value(49, &"TESTBUY1".to_string());
        fix_builder.with_value(49, &"TESTBUY2".to_string());
        fix_builder.with_value(52, &"20180920-18:24:59.643".to_string());
        fix_builder.with_value(56, &"TESTSELL1".to_string());
        fix_builder.with_value(98, &"0".to_string());
        fix_builder.with_value(108, &"60".to_string());
        let fix_to_assert = fix_builder.as_bytes();

        assert_eq!(fix_string, fix_to_assert.as_slice());
    }

    #[test]
    fn test_get_few_values_with_same_tag() {
        let fix_string = b"8=FIX.4.49=8735=A34=109249=TESTBUY149=TESTBUY252=20180920-18:24:59.64356=TESTSELL198=0108=6010=194";

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(34, &"1092".to_string());
        fix_builder.with_value(49, &"TESTBUY1".to_string());
        fix_builder.with_value(49, &"TESTBUY2".to_string());
        fix_builder.with_value(52, &"20180920-18:24:59.643".to_string());
        fix_builder.with_value(56, &"TESTSELL1".to_string());
        fix_builder.with_value(98, &"0".to_string());
        fix_builder.with_value(108, &"60".to_string());
        let fix_to_assert = fix_builder.as_bytes();

        assert_eq!(fix_string, fix_to_assert.as_slice());
        let tag49 = fix_builder.get_values_string("49");
        assert_eq!(2, tag49.len());
        assert_eq!("TESTBUY1", tag49[0]);
        assert_eq!("TESTBUY2", tag49[1]);
    }
}
