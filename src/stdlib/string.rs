pub struct BlazeString {
    bytes: crate::stdlib::collections::Vec<u8>,
}

impl BlazeString {
    pub fn new() -> Self {
        Self {
            bytes: crate::stdlib::collections::Vec::new(),
        }
    }
}
