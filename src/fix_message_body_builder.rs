use crate::utils::{FIX_BODY_LEN, FIX_VERSION};

#[derive(Clone)]
pub struct FixMessageBodyBuilder {
    data: Vec<u8>,
}

impl FixMessageBodyBuilder {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn append(&mut self, key: &str, value: &str) {
        crate::utils::write_fix_chunk(&mut self.data, key, value);
    }

    pub fn get_checksum(&self, fix_version: &str) -> String {
        let mut to_calc_check_sum = Vec::new();
        crate::utils::write_fix_chunk(&mut to_calc_check_sum, FIX_VERSION, fix_version);
        crate::utils::write_fix_chunk(
            &mut to_calc_check_sum,
            FIX_BODY_LEN,
            self.data.len().to_string().as_str(),
        );

        to_calc_check_sum.extend_from_slice(&self.data);

        crate::utils::calculate_check_sum(to_calc_check_sum.as_slice())
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
