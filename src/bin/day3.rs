#![feature(portable_simd)]

use aoc2025::{measure, NumberExt, Task};

fn main() {
    let input = include_bytes!("../../inputs/day3.txt");

    measure(Task::Part1, || part1(input.split(|c| *c == b'\n').filter(|v| v.len() == 100)));
    measure(Task::Part1, || part2(input.split(|c| *c == b'\n').filter(|v| v.len() == 100)));
}

fn part1<'a>(input: impl Iterator<Item=&'a[u8]>) -> u16 {
    let mut buffer = [0u8; 100];
    let mut total_power = 0;
    for line in input {
        buffer.copy_from_slice(line);
        total_power += battery_power_part1(&buffer);
    }

    total_power
}

fn part2<'a>(input: impl Iterator<Item=&'a[u8]>) -> u64 {
    let mut buffer = [0u8; 100];
    let mut total_power = 0;
    for line in input {
        buffer.copy_from_slice(line);
        total_power += battery_power_part2(&buffer);
    }

    total_power
}

fn battery_power_part1<const T: usize>(bytes: &[u8; T]) -> u16 {
    let max_byte = bytes.iter().max().copied().unwrap_or_default();
    let split_position = bytes.iter().position(|v| *v == max_byte).unwrap_or_default();
    match (bytes[..split_position].iter().max(), bytes[split_position + 1..].iter().max()) {
        (_, Some(right)) => u16::from_bytes(&[max_byte, *right]),
        (Some(left), _) => u16::from_bytes(&[*left, max_byte]),
        (None, None) => 0
    }
}

pub fn battery_power_part2<const T: usize>(bytes: &[u8; T]) -> u64 {
    let mut head = 0;
    let mut number = [0u8; 12];
    for index in  0..12 {
        let ((candidate, shift), _) = bytes[head..=T-(12-index)].iter().fold(
            ((0, 0), 0u8),
            |((max, position), offset), value| {
                let (max, position) = if *value > max {
                    (*value, offset + 1)
                } else {
                    (max, position)
                };

                ((max, position), offset + 1)
            }
        );
        number[index] = candidate;
        head += shift as usize;
    }

    u64::from_bytes(&number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_max_battery_power_part1() {
        assert_eq!(battery_power_part1(b"987654321111111"), 98);
        assert_eq!(battery_power_part1(b"811111111111119"), 89);
        assert_eq!(battery_power_part1(b"234234234234278"), 78);
        assert_eq!(battery_power_part1(b"818181911112111"), 92);
    }

    #[test]
    fn find_max_battery_power_part2() {
        assert_eq!(battery_power_part2(b"987654321111111"), 987654321111);
        assert_eq!(battery_power_part2(b"811111111111119"), 811111111119);
        assert_eq!(battery_power_part2(b"234234234234278"), 434234234278);
        assert_eq!(battery_power_part2(b"818181911112111"), 888911112111);
    }
}
