#![feature(portable_simd)]
#![feature(unboxed_closures)]

use std::ops::{Index, IndexMut, Range};
use std::simd::prelude::*;
use aoc2025::{line, NumberExt};

#[derive(Debug, PartialEq)]
enum Op {
    Multiply([u64; 16]),
    Add([u64; 16]),
}

impl Op {
    fn add() -> Self {
        Self::Add([0; 16])
    }

    fn mul() -> Self {
        Self::Multiply([1; 16])
    }

    #[cfg(test)]
    fn from_slice(mut self, slice: &[u64]) -> Self {
        assert!(slice.len() < 16);
        self.mut_slice()[..slice.len()].copy_from_slice(slice);
        self
    }

    fn mut_slice(&mut self) -> &mut [u64] {
        match self {
            Self::Multiply(m) | Self::Add(m) => m
        }
    }

    fn slice(&self) -> &[u64] {
        match self {
            Self::Multiply(m) | Self::Add(m) => m
        }
    }
}

impl Index<usize> for Op {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.slice()[index]
    }
}

impl IndexMut<usize> for Op {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.mut_slice()[index]
    }
}

impl Op {
    fn process(self) -> u64
    {
        match self {
            Op::Multiply(values) => values.into_iter().fold(1, u64::wrapping_mul),
            Op::Add(values) => values.into_iter().fold(0, u64::wrapping_add)
        }
    }
}

fn main() {
    let input = include_bytes!("../../inputs/day6.txt");
    println!("Part One: {}", part_one::<4>(input));
    println!("Part Two: {}", part_two::<4>(input));
}

fn part_one<const N: usize>(input: &[u8]) -> u64
{
    let columns = parse_part1::<N>(input);
    columns.into_iter().fold(0, |acc, col| col.process() + acc)
}

fn part_two<const N: usize>(input: &[u8]) -> u64
{
    let columns = parse_part2::<N>(input);
    columns.into_iter().fold(0, |acc, col| col.process() + acc)
}

const SPACE: u8x64 = u8x64::splat(b' ');

fn parse_part1<const N: usize>(mut input: &[u8]) -> Vec<Op> {
    let mut lines = [[0u8; 4096]; N];
    for row in 0..N {
        let (line, remainder) = line(input);
        lines[row][..line.len()].copy_from_slice(line);
        input = remainder;
    }
    let (input, _) = line(input);
    let mut result = Vec::new();
    for (byte, range) in frames(input) {
        let mut column = match byte {
            b'*' => Op::mul(),
            b'+' => Op::add(),
            _ => unreachable!()
        };

        let mut index = 0;
        for row in 0..N {
            let range = range.clone();
            column[index] = u64::from_bytes(&lines[row][range]);
            index += 1;
        }

        result.push(column);
    }

    result
}

fn parse_part2<const N: usize>(mut input: &[u8]) -> Vec<Op> {
    let mut lines = [[0u8; 4096]; N];
    for row in 0..N {
        let (line, remainder) = line(input);
        lines[row][..line.len()].copy_from_slice(line);
        input = remainder;
    }
    let (input, _) = line(input);
    let mut result = Vec::new();
    for (byte, range) in frames(input) {
        let mut column = match byte {
            b'*' => Op::mul(),
            b'+' => Op::add(),
            _ => unreachable!()
        };

        let mut index = 0;
        for col in range.rev() {
            let number = (0..N)
                .map(|v|lines[v][col])
                .filter(|v| *v >= b'0')
                .fold(0, |acc, v| acc * 10 + (v - b'0') as u64);

            column[index] = number;
            index += 1;
        }

        result.push(column);
    }

    result
}


fn frames(mut line: &[u8]) -> Vec<(u8, Range<usize>)> {
    let mut position = 0;
    let mut frames = Vec::<(u8, Range<usize>)>::with_capacity(line.len() / 3);
    while line.len() > 0 {
        let scan = u8x64::load_or(line, SPACE);
        let mut spaces = scan.simd_eq(SPACE).to_bitmask();

        while spaces.trailing_zeros() < 64 {
            let bytes_to_read = spaces.trailing_zeros() as usize;
            spaces >>= spaces.trailing_zeros();
            let bytes_to_skip = (bytes_to_read + spaces.trailing_ones() as usize).min(line.len());
            let last_byte = if line.len() - bytes_to_skip > 0 {
                (bytes_to_read + spaces.trailing_ones() as usize - 1).min(line.len())
            } else {
                bytes_to_read + line.len() - 1
            };

            spaces >>= spaces.trailing_ones();

            if spaces.trailing_zeros() == 64 && line.len() != bytes_to_skip {
                break;
            }

            let char = *&line[0];
            frames.push((
                char, position..position + last_byte,
            ));
            position += bytes_to_skip;
            line = &line[bytes_to_skip..];
        }
    }

    frames
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_columns_part1() {
        let input = b"123 3289  51 64
 45 64   387 23
  6 98   215 314
*   +    *   +  ";

        assert_eq!(
            parse_part1::<3>(input),
            vec![
                Op::mul().from_slice(&[123, 45, 6]),
                Op::add().from_slice(&[3289, 64, 98]),
                Op::mul().from_slice(&[51, 387, 215]),
                Op::add().from_slice(&[64, 23, 314]),
            ],
        );
    }

    #[test]
    fn parsing_columns_part2() {
        let input = b"123 328  51 64 \n 45 64  387 23 \n  6 98  215 314\n*   +   *   +  \n";

        assert_eq!(
            parse_part2::<3>(input),
            vec![
                Op::mul().from_slice(&[356, 24, 1]),
                Op::add().from_slice(&[8, 248, 369]),
                Op::mul().from_slice(&[175, 581, 32]),
                Op::add().from_slice(&[4, 431, 623]),
            ],
        );
    }

    #[test]
    fn parsing_columns_part2_with_longer_tail() {
        let input = b"123 328  51 64  \n 45 64  387 23  \n  6 98  215 3145\n*   +   *   +   \n";

        assert_eq!(
            parse_part2::<3>(input),
            vec![
                Op::mul().from_slice(&[356, 24, 1]),
                Op::add().from_slice(&[8, 248, 369]),
                Op::mul().from_slice(&[175, 581, 32]),
                Op::add().from_slice(&[5, 4, 431, 623]),
            ],
        );
    }


    #[test]
    fn part_one_test_case() {
        let input = b"123 328  51 64
 45 64  387 23
  6 98  215 314
*   +   *   +  ";

        assert_eq!(part_one::<3>(input), 4277556);
    }


    #[test]
    fn part_two_test_case() {
        let input = b"123 328  51 64
 45 64  387 23
  6 98  215 314
*   +   *   +  ";

        assert_eq!(part_two::<3>(input), 3263827);
    }


    #[test]
    fn parsing_lines() {
        let mut input = [b' ';200];
        input[121] = b'\n';
        input[160] = b'\n';

        assert_eq!(
            line(&input),
            (&input[..121], &input[122..])
        );
    }

    #[test]
    fn parsing_frames() {
        let input = b"*   *   *  +  *  *   *   +  +   +    +    +   +  +    *   +  +   +  +    *  +   ";

        assert_eq!(
            frames(input),
            vec![
                (b'*', 0..3),
                (b'*', 4..7),
                (b'*', 8..10),
                (b'+', 11..13),
                (b'*', 14..16),
                (b'*', 17..20),
                (b'*', 21..24),
                (b'+', 25..27),
                (b'+', 28..31),
                (b'+', 32..36),
                (b'+', 37..41),
                (b'+', 42..45),
                (b'+', 46..48),
                (b'+', 49..53),
                (b'*', 54..57),
                (b'+', 58..60),
                (b'+', 61..64),
                (b'+', 65..67),
                (b'+', 68..72),
                (b'*', 73..75),
                (b'+', 76..80),
            ]
        );
    }
}
