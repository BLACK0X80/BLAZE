use crate::stdlib::prelude::*;
use crate::stdlib::collections::vec::Vec;

pub struct String {
    bytes: Vec<u8>,
}

impl String {
    pub fn new() -> Self {
        String { bytes: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        String {
            bytes: Vec::with_capacity(capacity),
        }
    }

    pub fn from(s: &str) -> Self {
        String {
            bytes: s.as_bytes().iter().cloned().collect(),
        }
    }

    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        for byte in s.bytes() {
            self.bytes.push(byte);
        }
    }

    pub fn push_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.bytes.push(byte);
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.bytes.is_empty() {
            return None;
        }

        let mut end = self.bytes.len() - 1;
        while end > 0 && (self.bytes[end] & 0xC0) == 0x80 {
            end -= 1;
        }

        let ch = std::str::from_utf8(&self.bytes[end..]).ok()?.chars().next()?;
        self.bytes.truncate(end);
        Some(ch)
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn chars(&self) -> std::str::Chars {
        self.as_str().chars()
    }

    pub fn split(&self, delimiter: char) -> Vec<String> {
        self.as_str()
            .split(delimiter)
            .map(String::from)
            .collect()
    }

    pub fn trim(&self) -> String {
        String::from(self.as_str().trim())
    }

    pub fn to_lowercase(&self) -> String {
        String::from(&self.as_str().to_lowercase())
    }

    pub fn to_uppercase(&self) -> String {
        String::from(&self.as_str().to_uppercase())
    }

    pub fn contains(&self, pat: &str) -> bool {
        self.as_str().contains(pat)
    }

    pub fn starts_with(&self, pat: &str) -> bool {
        self.as_str().starts_with(pat)
    }

    pub fn ends_with(&self, pat: &str) -> bool {
        self.as_str().ends_with(pat)
    }

    pub fn replace(&self, from: &str, to: &str) -> String {
        String::from(&self.as_str().replace(from, to))
    }
}

impl Clone for String {
    fn clone(&self) -> Self {
        String {
            bytes: self.bytes.clone(),
        }
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for String {}

impl Hash for String {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

