#![feature(portable_simd)]
use std::simd::prelude::*;
use aoc2025::line;

fn main() {
    let input = include_bytes!("../../inputs/day7.txt");

    println!("Part1 Result: {}", part1(input));
    println!("Part2 Result: {}", part2(input));
}

const BEAM: u8 = b'|';
const START: u8 = b'S';
const SPLITTER: u8 = b'^';
const EMPTY: u8 = b'.';
const SPLITTER_PATTERN: u8x64 = u8x64::splat(SPLITTER);
const BEAM_PATTERN: u8x64 = u8x64::splat(BEAM);
const DEFAULT_EMPTY: u8x64 = u8x64::splat(EMPTY);

fn part1(input: &[u8]) -> u64 {
    let mut count= 0;
    process_beam(input, |_,_,_| count += 1);
    count
}

fn part2(input: &[u8]) -> u64 {
    let mut counts= [0; 256];

    process_beam(input, |left,from,right| {
        if counts[from] == 0 {
            counts[from] = 1;
        }

        counts[left] += counts[from];
        counts[right] += counts[from];
        counts[from] = 0;
    });

    counts.iter().sum()
}

fn process_beam<F: FnMut(usize, usize, usize)>(
    mut input: &[u8],
    mut split_beam: F) {
    // Skip first line as it is always middle
    let (first_line, remainder) = line(input);
    //println!("{}", String::from_utf8_lossy(&first_line));
    let mut beam_line = [EMPTY; 256];
    beam_line[..first_line.len()].copy_from_slice(first_line);
    match beam_line.iter().position(|&b| b == START) {
        Some(position) => beam_line[position] = BEAM,
        None => (),
    }
    input = remainder;

    while input.len() > 0 {
        let (mut current_line, remainder) = line(input);
        let mut debug_line = [EMPTY; 256];
        let current_beam_line = beam_line;
        debug_line[..current_line.len()].copy_from_slice(current_line);
        input = remainder;
        let mut offset = 0;
        while current_line.len() > 0 {
            let next_chunk = 64.min(current_line.len());
            let mut beam = beam_is_split(
                &current_line[..next_chunk],
                &current_beam_line[offset..offset+next_chunk.min(beam_line.len())],
            );

            let mut splitter = 0;
            while beam.trailing_zeros() < 64 {
                splitter += beam.trailing_zeros() as usize;
                beam >>= beam.trailing_zeros();
                if beam.trailing_ones() > 0 {
                    beam >>= 1;
                }
                let index = offset + splitter;
                beam_line[index - 1..=index+1].copy_from_slice(&[BEAM, EMPTY, BEAM]);
                split_beam(index - 1, index, index+1);
                splitter+=1;
            }

            current_line=&current_line[next_chunk..];
            offset+= next_chunk;
        }
        for index in 0..debug_line.len() {
            if current_beam_line[index] == BEAM && debug_line[index] == EMPTY {
                debug_line[index] = BEAM;
            }
        }
        //println!("{}", String::from_utf8_lossy(&debug_line[..first_line.len()]));
    }
}


fn beam_is_split(line: &[u8], beam: &[u8]) -> u64 {
    (u8x64::load_or(line, DEFAULT_EMPTY).simd_eq(SPLITTER_PATTERN) &
    u8x64::load_or(beam, DEFAULT_EMPTY).simd_eq(BEAM_PATTERN)).to_bitmask()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_test_case() {
        let input = b".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............";

        assert_eq!(part1(input), 21);
    }

    #[test]
    fn part2_test_case() {
        let input = b".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............";

        assert_eq!(part2(input), 40);
    }
}
