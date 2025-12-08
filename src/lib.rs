#![feature(portable_simd)]
use std::ops::{AddAssign, MulAssign};
use std::simd::prelude::*;

pub trait NumberExt {
    fn from_bytes(slice: &[u8]) -> Self;
}

impl <T: MulAssign + AddAssign + From<u8> + Copy> NumberExt for T {
    fn from_bytes(slice: &[u8]) -> Self {
        let mut accum = 0u8.into();
        let decimal = 10u8.into();
        for byte in slice {
            if *byte < b'0' {
                continue
            }
            accum *= decimal;
            accum += (byte - b'0').into();
        }

        accum
    }
}


const NEW_LINES: u8x64 = u8x64::splat(b'\n');

pub fn line(input: &[u8]) -> (&[u8], &[u8]) {
    let mut new_line_pos = 0;
    for scan in input.chunks(64) {
        let scan = u8x64::load_or_default(scan);
        let new_line = scan.simd_eq(NEW_LINES).first_set();
        if let Some(new_line) = new_line {
            return (&input[..new_line_pos + new_line], &input[new_line_pos + new_line + 1..]);
        }
        new_line_pos += 64;
    };

    (input, &[])
}

pub struct Lines<'a> {
    input: &'a[u8]
}

pub fn lines(input: &[u8]) -> Lines<'_> {
    Lines { input }
}

impl <'a> Iterator for Lines<'a> {
    type Item = &'a[u8];

    fn next(&mut self) -> Option<Self::Item> {
        match self.input.len() {
            0 => None,
            _ => {
                let (line, remainder) = line(self.input);
                self.input = remainder;
                Some(line)
            }
        }
    }
}

pub fn digits(value: u64) -> u8 {
    value.checked_ilog10().unwrap_or(0) as u8 + 1
}

pub fn factors(value: u8) -> impl Iterator<Item=u8> {
    (2..value).filter_map(move |factor| if value % factor == 0 { Some(factor) } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_number_of_digits() {
        assert_eq!(digits(10), 2);
        assert_eq!(digits(10000),5);
        assert_eq!(digits(0), 1);
        assert_eq!(digits(1), 1);
    }

    #[test]
    fn finds_factors_of_simple_int() {
        assert_eq!(factors(1).collect::<Vec<_>>(), vec![]);
        assert_eq!(factors(4).collect::<Vec<_>>(), vec![2]);
        assert_eq!(factors(6).collect::<Vec<_>>(), vec![2, 3]);
        assert_eq!(factors(8).collect::<Vec<_>>(), vec![2, 4]);
        assert_eq!(factors(9).collect::<Vec<_>>(), vec![3]);
    }

    #[test]
    fn builds_numbers() {
        assert_eq!(u8::from_bytes(b"123"), 123);
        assert_eq!(u16::from_bytes(b"1023"), 1023);
        assert_eq!(u32::from_bytes(b"1023123123"), 1023123123);
        assert_eq!(u32::from_bytes(b"   1023123123  "), 1023123123);
        assert_eq!(u32::from_bytes(b"   1023123123"), 1023123123);
        assert_eq!(u32::from_bytes(b"1023123123   "), 1023123123);
    }

    #[test]
    fn lines_iterator() {
        let mut lines = lines(b"first\nsecond\nthird");

        assert_eq!(lines.next(), Some(b"first".as_ref()));
        assert_eq!(lines.next(), Some(b"second".as_ref()));
        assert_eq!(lines.next(), Some(b"third".as_ref()));
        assert_eq!(lines.next(), None)
    }
}
