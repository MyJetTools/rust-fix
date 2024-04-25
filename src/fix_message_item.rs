#[derive(Debug)]
pub struct FixMessageItem<'s> {
    pub key: &'s str,
    pub value: &'s str,
}

impl<'s> FixMessageItem<'s> {
    pub fn from_str(src: &'s str) -> Self {
        let index = src.find('=');

        if index.is_none() {
            panic!("Invalid fix item: {}", src);
        }

        let index = index.unwrap();

        let key = &src[..index];
        let mut value = &src[index + 1..];

        if *value.as_bytes().last().unwrap() == b'|' {
            value = &value[..value.len() - 1];
        }
        Self { key, value }
    }

    pub fn from_slice(src: &'s [u8]) -> Self {
        let index = find_index(src);

        let index = index.unwrap();

        let key = &src[..index];
        let mut value = &src[index + 1..];

        if *value.last().unwrap() == 1 {
            value = &value[..value.len() - 1];
        }
        Self {
            key: std::str::from_utf8(key).unwrap(),
            value: std::str::from_utf8(value).unwrap(),
        }
    }
}

fn find_index(src: &[u8]) -> Option<usize> {
    for (index, byte) in src.iter().enumerate() {
        if *byte == b'=' {
            return Some(index);
        }
    }
    None
}

#[cfg(test)]
mod test {
    use crate::FixMessageItem;

    #[test]
    fn test_parsing_from_bytes_with_ending() {
        let mut src = "9=123".as_bytes().to_vec();
        src.push(1);

        let item = FixMessageItem::from_slice(src.as_slice());

        assert_eq!(item.key, "9");
        assert_eq!(item.value, "123");
    }

    #[test]
    fn test_parsing_from_bytes_with_no_ending() {
        let src = "9=123".as_bytes().to_vec();

        let item = FixMessageItem::from_slice(src.as_slice());

        assert_eq!(item.key, "9");
        assert_eq!(item.value, "123");
    }

    #[test]
    fn test_parsing_from_str_with_ending() {
        let mut src = "9=123".to_string();
        src.push('|');

        let item = FixMessageItem::from_str(src.as_str());

        assert_eq!(item.key, "9");
        assert_eq!(item.value, "123");
    }

    #[test]
    fn test_parsing_from_str_with_no_ending() {
        let src = "9=123";

        let item = FixMessageItem::from_str(src);

        assert_eq!(item.key, "9");
        assert_eq!(item.value, "123");
    }
}
