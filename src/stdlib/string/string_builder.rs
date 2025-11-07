use super::super::collections::vec::Vec;

pub struct StringBuilder {
    buffer: Vec<u8>,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn append(&mut self, s: &str) {
        for &byte in s.as_bytes() {
            self.buffer.push(byte);
        }
    }

    pub fn append_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.append(s);
    }

    pub fn append_int(&mut self, n: i64) {
        if n == 0 {
            self.append_char('0');
            return;
        }

        let mut num = n;
        let negative = num < 0;
        if negative {
            num = -num;
        }

        let mut digits = Vec::new();
        while num > 0 {
            digits.push((num % 10) as u8 + b'0');
            num /= 10;
        }

        if negative {
            self.append_char('-');
        }

        while let Some(digit) = digits.pop() {
            self.buffer.push(digit);
        }
    }

    pub fn append_float(&mut self, f: f64) {
        let s = format!("{}", f);
        self.append(&s);
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for i in 0..self.buffer.len() {
            if let Some(&byte) = self.buffer.get(i) {
                if let Ok(s) = std::str::from_utf8(&[byte]) {
                    result.push_str(s);
                }
            }
        }
        result
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
}

impl From<&str> for StringBuilder {
    fn from(s: &str) -> Self {
        let mut builder = StringBuilder::with_capacity(s.len());
        builder.append(s);
        builder
    }
}

impl std::fmt::Display for StringBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct BlazeString {
    data: Vec<u8>,
}

impl BlazeString {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        let mut data = Vec::with_capacity(s.len());
        for &byte in s.as_bytes() {
            data.push(byte);
        }
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn push_str(&mut self, s: &str) {
        for &byte in s.as_bytes() {
            self.data.push(byte);
        }
    }

    pub fn push(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.push_str(s);
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        std::str::from_utf8(self.as_bytes())
            .unwrap_or("")
            .chars()
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.data.get(0).unwrap_or(&0) as *const u8, self.data.len())
        }
    }

    pub fn to_std_string(&self) -> String {
        String::from_utf8_lossy(self.as_bytes()).into_owned()
    }

    pub fn contains(&self, pattern: &str) -> bool {
        self.to_std_string().contains(pattern)
    }

    pub fn starts_with(&self, pattern: &str) -> bool {
        self.to_std_string().starts_with(pattern)
    }

    pub fn ends_with(&self, pattern: &str) -> bool {
        self.to_std_string().ends_with(pattern)
    }

    pub fn split(&self, delimiter: char) -> std::vec::Vec<BlazeString> {
        let s = self.to_std_string();
        s.split(delimiter)
            .map(|part| BlazeString::from_str(part))
            .collect()
    }

    pub fn trim(&self) -> BlazeString {
        let s = self.to_std_string();
        BlazeString::from_str(s.trim())
    }

    pub fn to_uppercase(&self) -> BlazeString {
        let s = self.to_std_string();
        BlazeString::from_str(&s.to_uppercase())
    }

    pub fn to_lowercase(&self) -> BlazeString {
        let s = self.to_std_string();
        BlazeString::from_str(&s.to_lowercase())
    }

    pub fn replace(&self, from: &str, to: &str) -> BlazeString {
        let s = self.to_std_string();
        BlazeString::from_str(&s.replace(from, to))
    }

    pub fn substring(&self, start: usize, end: usize) -> BlazeString {
        let s = self.to_std_string();
        if start >= s.len() || end > s.len() || start >= end {
            return BlazeString::new();
        }
        BlazeString::from_str(&s[start..end])
    }
}

impl std::fmt::Display for BlazeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_std_string())
    }
}

impl std::fmt::Debug for BlazeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlazeString(\"{}\")", self.to_std_string())
    }
}

impl PartialEq for BlazeString {
    fn eq(&self, other: &Self) -> bool {
        self.data.len() == other.data.len() && 
            (0..self.data.len()).all(|i| {
                self.data.get(i) == other.data.get(i)
            })
    }
}

impl Eq for BlazeString {}

impl Clone for BlazeString {
    fn clone(&self) -> Self {
        let mut data = Vec::with_capacity(self.data.len());
        for i in 0..self.data.len() {
            if let Some(&byte) = self.data.get(i) {
                data.push(byte);
            }
        }
        Self { data }
    }
}
