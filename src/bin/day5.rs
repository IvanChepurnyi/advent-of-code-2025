#![feature(portable_simd)]

use std::cmp::Ordering;
use std::ops::RangeInclusive;
use std::simd::prelude::*;
use aoc2025::NumberExt;

#[derive(Debug, PartialEq, Eq)]
struct SimdRange
{
    start: u64x8,
    end: u64x8,
    range: RangeInclusive<u64>
}


impl SimdRange {
    fn new(start: u64, end: u64) -> SimdRange {
        Self {
            start: u64x8::splat(start),
            end: u64x8::splat(end),
            range: start..=end
        }
    }

    fn match_numbers(&self, slice: u64x8) -> mask64x8
    {
        slice.simd_gt(self.start) & slice.simd_le(self.end)
    }

    fn range(&self) -> RangeInclusive<u64> {
        self.range.clone()
    }
}

impl PartialOrd<SimdRange> for SimdRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SimdRange {
    fn cmp(&self, other: &Self) -> Ordering {
        let (start, end) = (*self.range.start(), *self.range.end());
        let (other_start, other_end) = (*other.range.start(), *other.range.end());
        match start.cmp(&other_start) {
            Ordering::Equal => end.cmp(&other_end),
            ord => ord
        }
    }
}

fn main() {
    let input = include_bytes!("../../inputs/day5.txt");

    let (ranges, numbers) = parse_input(input);

    println!("Part 1: {}", part1(&ranges, &numbers));
    println!("Part 2: {}", part2(&ranges));
}

const RANGE_SPLIT: u8x32 = u8x32::splat(b'-');

fn parse_input(input: &[u8]) -> (Vec<SimdRange>, Vec<u64>)
{
    let (mut ranges, input) = input.split(|c| *c == b'\n')
        .filter(|line| !line.is_empty())
        .fold(
            (Vec::new(), Vec::new()),
            |(mut ranges, mut numbers), line| {
                let check_range = u8x32::load_or_default(line);
                let first_split = check_range.simd_eq(RANGE_SPLIT);
                match first_split.first_set() {
                    Some(range_split) => {
                        let left = u64::from_bytes(&line[..range_split]);
                        let right = u64::from_bytes(&line[range_split+1..]);

                        ranges.push(SimdRange::new(left, right));
                    },
                    None => {
                        numbers.push(u64::from_bytes(line));
                    }
                }
                (ranges, numbers)
            });

    ranges.sort();
    (ranges, input)
}

fn part1(ranges: &[SimdRange], numbers: &[u64]) -> u32
{
    let chunks = numbers.chunks_exact(8);
    let remainder = chunks.remainder();

    chunks.fold(0, |acc, chunk| {
        acc + valid_numbers(chunk, ranges)
    }) + valid_numbers(remainder, ranges)
}

fn part2(ranges: &[SimdRange]) -> u64
{
    let final_ranges = merge_ranges(ranges);

    final_ranges.into_iter()
        .fold(0, |acc, range| acc + (*range.end() - *range.start()) + 1)
}

fn valid_numbers(input: &[u64], ranges: &[SimdRange]) -> u32 {
    let chunk = u64x8::load_or_default(input);
    let valid = ranges.iter()
        .fold(
            mask64x8::splat(false),
            move |acc, range| acc | range.match_numbers(chunk)
        );

    valid.to_bitmask().count_ones()
}

fn merge_ranges(ranges: &[SimdRange]) -> Vec<RangeInclusive<u64>>
{
    let mut result: Vec<RangeInclusive<u64>> = ranges.iter().map(SimdRange::range).collect::<Vec<_>>();

    let mut index = 0;

    while index < result.len() - 1 {
        let [first, second] = &mut result[index..=index+1] else { unreachable!() };

        if first.contains(second.end()) {
            result.remove(index + 1);
            continue;
        }

        if *second.start() <= first.end() + 1 {
            *first = *first.start()..=*second.end();
            result.remove(index + 1);
            continue;
        }
        index+=1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input() {
        let input = b"3-5
10-14
16-20
12-18

1
5
8
11
17
32";

        let (ranges, numbers) = parse_input(input);

        assert_eq!(
            ranges,
            vec![
                SimdRange::new(3,5),
                SimdRange::new(10,14),
                SimdRange::new(12,18),
                SimdRange::new(16,20),
            ]
        );
        assert_eq!(
            numbers,
            vec![1, 5, 8, 11, 17, 32]
        );
    }

    #[test]
    fn test_simd_range_on_slice() {
        assert_eq!(
            SimdRange::new(3,5)
            .match_numbers(
                u64x8::from_array([1, 5, 8, 11, 17, 32, 0, 0])
            ),
            mask64x8::from_array(
                [false, true, false, false, false, false, false, false]
            )
        );
    }

    #[test]
    fn part1_test_case() {
        assert_eq!(
            part1(&vec![
            SimdRange::new(3,5),
            SimdRange::new(10,14),
            SimdRange::new(16,20),
            SimdRange::new(12,18)
        ], &vec![1, 5, 8, 11, 17, 32]), 3);
    }

    #[test]
    fn part2_deduplicate_ranges() {

        assert_eq!(
            merge_ranges(&vec![
                SimdRange::new(3,5),
                SimdRange::new(10,14),
                SimdRange::new(12,18),
                SimdRange::new(16,20),
            ]),
            vec![
                3..=5,
                10..=20
            ]
        )
    }

    #[test]
    fn part2_test_case() {

        assert_eq!(
            part2(&vec![
                SimdRange::new(3,5),
                SimdRange::new(10,14),
                SimdRange::new(12,18),
                SimdRange::new(16,20),
            ]),
            14
        )
    }
}