#![feature(portable_simd)]

use std::ops::{Add, Mul};
use std::simd::prelude::*;
use aoc2025::{lines, measure, Lines, NumberExt, Task};

fn main() {
    let input = include_bytes!("../../inputs/day9.txt");

    measure(Task::Part1, || part1(Parser::new(input)));
}

const ONES: u64x16 = u64x16::splat(1);

fn part1(input: Parser) -> u64 {
    let coords: (Vec<_>, Vec<_>) = input.unzip();
    let items = 0..coords.0.len();
    let mut best_rectangle = 0;

    for current_item in items {
        let (current_x, current_y) = (
            u64x16::splat(coords.0[current_item]),
            u64x16::splat(coords.1[current_item])
        );

        let mut chunk_x = coords.0.chunks(16);
        let mut chunk_y = coords.1.chunks(16);

        while let (Some(chunk_x), Some(chunk_y)) = (chunk_x.next(), chunk_y.next()) {
            let len_x = u64x16::load_or(chunk_x, current_x).abs_diff(current_x).add(ONES);
            let len_y = u64x16::load_or(chunk_y, current_y).abs_diff(current_y).add(ONES);
            let area = len_x * len_y;
            best_rectangle = best_rectangle.max(area.reduce_max())
        }
    }

    best_rectangle
}


#[derive(Clone,Copy)]
struct Parser<'a> {
    lines: Lines<'a>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            lines: lines(input),
        }
    }
}

const PATTERN: u8x16 = u8x16::splat(b',');

impl Iterator for Parser<'_> {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(line) => {
                let scan = u8x16::load_or_default(&line[..]).simd_eq(PATTERN).to_bitmask();
                if scan == 0 {
                    return None;
                }
                let position = scan.trailing_zeros() as usize;
                Some((
                    u64::from_bytes(&line[..position]),
                    u64::from_bytes(&line[position+1..])
                ))
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> Parser<'static> {
        Parser::new(b"7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3")
    }

    #[test]
    fn test_case_part1() {
        assert_eq!(
            part1(test_data()),
            50
        );
    }
}
