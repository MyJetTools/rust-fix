use crate::{FixMessageItem, FixSerializeError};

pub struct FixMessageIterator<'s> {
    data: &'s [u8],
    current_index: usize,
    delimiter: u8,
}

impl<'s> FixMessageIterator<'s> {
    pub fn from_str(data: &'s str) -> Self {
        Self {
            data: data.as_bytes(),
            current_index: 0,
            delimiter: b'|',
        }
    }

    pub fn from_slice(data: &'s [u8]) -> Self {
        Self {
            data,
            current_index: 0,
            delimiter: 1,
        }
    }
}

impl<'s> Iterator for FixMessageIterator<'s> {
    type Item = Result<FixMessageItem<'s>, FixSerializeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.data.len() {
            return None;
        }

        let start = self.current_index;

        while self.data[self.current_index] != self.delimiter {
            self.current_index += 1;

            if self.current_index >= self.data.len() {
                return Some(Err(FixSerializeError::FixDelimiterNotFound));
            }
        }

        let item = &self.data[start..self.current_index];

        let result = if self.delimiter == 1 {
            FixMessageItem::from_slice(item)
        } else {
            FixMessageItem::from_str(std::str::from_utf8(item).unwrap())
        };

        self.current_index += 1;

        Some(Ok(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::FixMessageIterator;

    #[test]
    fn test_iterator() {
        let src_string = "8=FIX.4.4|9=75|35=A|34=1092|49=TESTBUY1|52=20180920-18:24:59.643|56=TESTSELL1|98=0|108=60|10=178|";

        let mut iterator = FixMessageIterator::from_str(src_string);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("8", next_item.key);
        assert_eq!("FIX.4.4", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("9", next_item.key);
        assert_eq!("75", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("35", next_item.key);
        assert_eq!("A", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("34", next_item.key);
        assert_eq!("1092", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("49", next_item.key);
        assert_eq!("TESTBUY1", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("52", next_item.key);
        assert_eq!("20180920-18:24:59.643", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("56", next_item.key);
        assert_eq!("TESTSELL1", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("98", next_item.key);
        assert_eq!("0", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("108", next_item.key);
        assert_eq!("60", next_item.value);

        let next_item = iterator.next().unwrap().unwrap();
        assert_eq!("10", next_item.key);
        assert_eq!("178", next_item.value);

        let itm = iterator.next();
        assert_eq!(true, itm.is_none());
    }
}
