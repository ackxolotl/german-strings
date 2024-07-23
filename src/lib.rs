use std::alloc::{alloc, dealloc, Layout};
use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::ptr;

pub struct GermanString {
    len: u32,
    data: [u8; 12],
}

impl GermanString {
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn prefix(&self) -> &[u8] {
        &self.data[..min(4, self.len) as usize]
    }

    pub fn as_str(&self) -> &str {
        let slice = if self.len <= 12 {
            &self.data[..self.len as usize]
        } else {
            let ptr = u64::from_le_bytes(self.data[4..].try_into().unwrap()) as *mut u8;
            unsafe { std::slice::from_raw_parts(ptr, self.len as usize) }
        };
        std::str::from_utf8(slice).unwrap()
    }
}

impl From<&str> for GermanString {
    fn from(s: &str) -> GermanString {
        let len = s.bytes().len();
        let mut data = [0; 12];

        if len <= 12 {
            data[..len].copy_from_slice(&s.as_bytes()[..len]);
        } else {
            // prefix
            data[..4].copy_from_slice(&s.as_bytes()[..4]);
            // copy s to new location
            let layout = Layout::array::<u8>(len).unwrap();
            let ptr = unsafe { alloc(layout) };
            unsafe { ptr::copy_nonoverlapping(s.as_ptr(), ptr, len) };
            // store ptr to new location
            data[4..].copy_from_slice(&(ptr as u64).to_le_bytes());
        }

        Self {
            len: len as u32,
            data,
        }
    }
}

impl Display for GermanString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.as_str())
    }
}

impl Drop for GermanString {
    fn drop(&mut self) {
        if self.len > 12 {
            let layout = Layout::array::<u8>(self.len as usize).unwrap();
            let ptr = u64::from_le_bytes(self.data[4..].try_into().unwrap()) as *mut u8;
            unsafe { dealloc(ptr, layout) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let g = GermanString::from("");
        assert_eq!(g.len(), 0);
        assert_eq!(g.is_empty(), true);
        assert_eq!(g.prefix(), b"");
        assert_eq!(g.as_str(), "");

        let g = GermanString::from("12characters");
        assert_eq!(g.len(), 12);
        assert_eq!(g.is_empty(), false);
        assert_eq!(g.prefix(), b"12ch");
        assert_eq!(g.as_str(), "12characters");

        let g = GermanString::from("123456789abcdefghijklmn");
        assert_eq!(g.len(), 23);
        assert_eq!(g.prefix(), b"1234");
        assert_eq!(g.as_str(), "123456789abcdefghijklmn");

        let g = GermanString::from("👩‍👩‍👧‍👦");
        assert_eq!(g.len(), 25);
        assert_eq!(g.prefix(), b"\xF0\x9F\x91\xA9");
        assert_eq!(g.as_str(), "👩‍👩‍👧‍👦");
    }
}
