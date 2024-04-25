use crate::FixMessageBodyBuilder;

//pub const FIX_VERSION: &str = "8";
//pub const FIX_BODY_LEN: &str = "9";
//pub const FIX_CHECK_SUM: &str = "10";
//pub const FIX_MESSAGE_TYPE: &str = "35";

#[derive(Clone)]
pub struct FixMessageWriter {
    fix_version: &'static str,
    body: FixMessageBodyBuilder,
}

impl FixMessageWriter {
    pub fn new(fix_version: &'static str, message_type: &str) -> Self {
        let mut body = FixMessageBodyBuilder::new();
        body.append(
            crate::utils::FIX_MESSAGE_TYPE,
            message_type.to_string().as_str(),
        );

        return Self { fix_version, body };
    }

    /*
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
     */

    /*
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
    */
    pub fn with_value(&mut self, key: &str, value: &str) {
        self.body.append(key, value);
    }

    /*
          fn with_value_as_bytes(&mut self, key: Vec<u8>, value: Vec<u8>) {
              self.data.push((key, value));
          }
    */
    pub fn compile_message(&self) -> Vec<u8> {
        let mut result = Vec::new();
        crate::utils::write_fix_chunk(&mut result, crate::utils::FIX_VERSION, self.fix_version);

        crate::utils::write_body_len(&mut result, self.body.len());
        result.extend_from_slice(self.body.as_slice());

        let check_sum = self.body.get_checksum(&self.fix_version);

        crate::utils::write_fix_chunk(&mut result, crate::utils::FIX_CHECK_SUM, check_sum.as_str());

        return result;
    }

    /*
    fn calculate_check_sum(&self) -> String {
        let mut result = String::new();
        write_fix_chunk(&mut result, FIX_VERSION, self.fix_version);

        write_fix_chunk(
            &mut result,
            FIX_BODY_LEN,
            self.body.len().to_string().as_str(),
        );

        result.push_str(self.body.as_str());

        return calculate_check_sum(result.as_bytes());
    }
     */
}

impl ToString for FixMessageWriter {
    fn to_string(&self) -> String {
        let result = self.compile_message();
        crate::utils::convert_fix_message_to_string(result)
    }
}

#[cfg(test)]
mod test {
    use crate::FixMessageReader;

    use super::*;

    #[test]
    fn test_to_fix_string() {
        let fix_string = "8=FIX.4.4|9=75|35=A|34=1092|49=TESTBUY1|52=20180920-18:24:59.643|56=TESTSELL1|98=0|108=60|10=178|";

        let mut fix_builder = FixMessageWriter::new("FIX.4.4", "A");
        fix_builder.with_value("34", &"1092".to_string());
        fix_builder.with_value("49", &"TESTBUY1".to_string());
        fix_builder.with_value("52", &"20180920-18:24:59.643".to_string());
        fix_builder.with_value("56", &"TESTSELL1".to_string());
        fix_builder.with_value("98", &"0".to_string());
        fix_builder.with_value("108", &"60".to_string());

        let fix_to_assert: String = fix_builder.to_string();

        assert_eq!(fix_string, &fix_to_assert);
    }

    #[test]
    fn test_to_bytes() {
        let fix_string = b"8=FIX.4.49=7535=A34=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0108=6010=178";

        let mut fix_builder = FixMessageWriter::new("FIX.4.4", "A");
        fix_builder.with_value("34", "1092");
        fix_builder.with_value("49", "TESTBUY1");
        fix_builder.with_value("52", "20180920-18:24:59.643");
        fix_builder.with_value("56", "TESTSELL1");
        fix_builder.with_value("98", "0");
        fix_builder.with_value("108", "60");

        let fix_to_assert = fix_builder.compile_message();

        assert_eq!(fix_string, fix_to_assert.as_slice());
    }

    #[test]
    fn test_few_values_with_same_tag() {
        let fix_string = b"8=FIX.4.49=8735=A34=109249=TESTBUY149=TESTBUY252=20180920-18:24:59.64356=TESTSELL198=0108=6010=194";

        let mut fix_builder = FixMessageWriter::new("FIX.4.4", "A");
        fix_builder.with_value("34", "1092");
        fix_builder.with_value("49", "TESTBUY1");
        fix_builder.with_value("49", "TESTBUY2");
        fix_builder.with_value("52", "20180920-18:24:59.643");
        fix_builder.with_value("56", "TESTSELL1");
        fix_builder.with_value("98", "0");
        fix_builder.with_value("108", "60");
        let fix_to_assert = fix_builder.compile_message();

        assert_eq!(fix_string, fix_to_assert.as_slice());
    }

    #[test]
    fn test_get_few_values_with_same_tag() {
        let fix_string = b"8=FIX.4.49=8735=A34=109249=TESTBUY149=TESTBUY252=20180920-18:24:59.64356=TESTSELL198=0108=6010=194";

        let mut fix_builder = FixMessageWriter::new("FIX.4.4", "A");
        fix_builder.with_value("34", "1092");
        fix_builder.with_value("49", "TESTBUY1");
        fix_builder.with_value("49", "TESTBUY2");
        fix_builder.with_value("52", "20180920-18:24:59.643");
        fix_builder.with_value("56", "TESTSELL1");
        fix_builder.with_value("98", "0");
        fix_builder.with_value("108", "60");
        let fix_to_assert = fix_builder.compile_message();

        assert_eq!(fix_string, fix_to_assert.as_slice());

        let fix_reader = FixMessageReader::from_bytes(fix_string);

        let tag49 = fix_reader.get_values("49").unwrap();
        assert_eq!(2, tag49.len());
        assert_eq!("TESTBUY1", tag49[0]);
        assert_eq!("TESTBUY2", tag49[1]);
    }
}
