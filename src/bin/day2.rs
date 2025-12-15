#![feature(slice_split_once)]

use std::ops::RangeInclusive;
use aoc2025::{digits, factors, measure, NumberExt, Task};

fn main() {
    let input = include_bytes!("../../inputs/day2.txt");

    measure(Task::Part1, || sum_invalid(input, invalid_id_part1));
    measure(Task::Part2, || sum_invalid(input, invalid_id_part2));
}

fn from_ranges(input: &[u8]) -> impl Iterator<Item=u64> {
    input.split(|char| char == &b',').filter_map(parse_range).flatten()
}

fn sum_invalid(input: &[u8], validator: fn(u64) -> Option<u64>) -> u64 {
    from_ranges(input).filter_map(validator).fold(0, |acc, value| acc + value)
}

fn parse_range(input: &[u8]) -> Option<RangeInclusive<u64>> {
    input.split_once(|v| *v == b'-').map(|(start, end)| {
        NumberExt::from_bytes(start)..=NumberExt::from_bytes(end)
    })
}

fn invalid_id_part1(number: u64) -> Option<u64> {
    let digits = digits(number);
    if digits % 2 == 1 {
        return None;
    }
    let split_point = digits / 2;
    let multiplier = 10u64.pow(split_point as u32);
    let left = number / multiplier;
    let right = number % multiplier;

    if left != right {
        return None;
    }

    Some(number)
}

fn invalid_id_part2(number: u64) -> Option<u64> {
    let digits = digits(number);


    if digits == 1 {
        return None;
    }

    let same_digit_number = ((10u64.pow(digits as u32) - 1) / 9) * (number % 10);

    if same_digit_number == number {
        return Some(number);
    }

    'outer: for factor in factors(digits) {
        let multiplier = 10u64.pow(factor as u32);
        let pattern = number % multiplier;
        let mut left_part = number / multiplier;

        while left_part > 0 {
            if left_part % multiplier != pattern {
                continue 'outer;
            }
            left_part /= multiplier;
        }
        return Some(number);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ranges_of_integers() {
        assert_eq!(
            from_ranges(b"10-15,20-28").collect::<Vec<_>>(),
            vec![10, 11, 12, 13, 14, 15, 20, 21, 22, 23, 24, 25, 26, 27, 28]
        );
    }

    #[test]
    fn tests_invalid_ids_part1() {
        assert_eq!(invalid_id_part1(1), None);
        assert_eq!(invalid_id_part1(10), None);
        assert_eq!(invalid_id_part1(11), Some(11));
        assert_eq!(invalid_id_part1(121), None);
        assert_eq!(invalid_id_part1(111), None);
        assert_eq!(invalid_id_part1(1212), Some(1212));
    }

    #[test]
    fn tests_invalid_ids_part2() {
        assert_eq!(invalid_id_part2(1), None);
        assert_eq!(invalid_id_part2(10), None);
        assert_eq!(invalid_id_part2(11), Some(11));
        assert_eq!(invalid_id_part2(121), None);
        assert_eq!(invalid_id_part2(111), Some(111));
        assert_eq!(invalid_id_part2(1212), Some(1212));
        assert_eq!(invalid_id_part2(121121121), Some(121121121));
    }

    #[test]
    fn part1_test_case() {
        assert_eq!(
            sum_invalid(b"11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124", invalid_id_part1),
            1227775554
        );
    }

    #[test]
    fn part2_test_case() {
        assert_eq!(
            sum_invalid(b"11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124", invalid_id_part2),
            4174379265
        );
    }
}
