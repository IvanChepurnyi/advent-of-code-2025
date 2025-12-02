use aoc2025::NumberExt;

fn main() {
    let input = include_bytes!("../../inputs/day1.txt");

    let slices = input.split(|&b| b == b'\n' || b == b' ');
    let numbers = slices.filter_map(parse_slice).collect::<Vec<_>>();
    let result_part1 = part1(&numbers);
    let result_part2 = part2(&numbers);

    println!("Result for Part 1: {}", result_part1);
    println!("Result for Part 2: {}", result_part2);
}

fn parse_slice(slice:&[u8]) -> Option<i16> {
    if slice.is_empty()  {
        return None;
    }

    match slice[0] {
        b'L' => Some(-i16::from_bytes(&slice[1..])),
        b'R' => Some(i16::from_bytes(&slice[1..])),
        _ => None
    }
}

pub fn part1(input: &[i16]) -> i16 {
    let mut dial: i32 = 50;
    let mut password = 0;

    for delta in input {
        dial += *delta as i32;
        password += if dial % 100 == 0 { 1 } else { 0 };
    }

    password as i16 
}

pub fn part2(input: &[i16]) -> i16 {
    let mut dial = 50;
    let mut password = 0;

    for delta in input {
        if *delta >= 0 {
            password += (dial + delta) / 100;
        } else {
            let reversed = (100 - dial) % 100;
            password += (reversed - delta) / 100;
        }
        dial = (dial + delta).rem_euclid(100);
    }

    password
}