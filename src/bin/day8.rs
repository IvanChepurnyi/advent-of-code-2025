#![feature(portable_simd)]
extern crate core;

use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::simd::prelude::*;
use aoc2025::{lines, Lines, NumberExt};

const PATTERN: u8x32 = u8x32::splat(b',');

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
struct Coordinate(u64, u64, u64);

impl Coordinate {
    fn distance(&self, other: &Coordinate) -> u64 {
        let left = u64x4::load_or_default(&[self.0, self.1, self.2]);
        let right = u64x4::load_or_default(&[other.0, other.1, other.2]);

        distance(&left, &right)
    }
}

#[derive(Clone,Copy)]
struct Parser<'a> {
    lines: Lines<'a>,
}

struct HeapWithLimit {
    items: BTreeMap<u64, (Coordinate, Coordinate)>
}

impl HeapWithLimit {
    fn new() -> Self {
        Self {
            items: BTreeMap::new()
        }
    }

    fn add(&mut self, distance: u64, left: Coordinate, right: Coordinate) {
        self.items.insert(distance, (left, right));
    }

    fn iter(&self) -> impl Iterator<Item=(Coordinate, Coordinate)> {
        self.items.iter().map(|(_k,v)| (v.0, v.1))
    }
}

impl Iterator for Parser<'_> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(mut line) => {
                let mut scan = u8x32::load_or_default(&line[..]).simd_eq(PATTERN).to_bitmask();
                if scan.count_ones() != 2 {
                    return None;
                }
                let mut coordinate = Coordinate::default();

                (coordinate.0, line) = read_int(line, &mut scan);
                (coordinate.1, line) = read_int(line, &mut scan);
                (coordinate.2, _) = read_int(line, &mut scan);

                Some(coordinate)
            },
            None => None,
        }
    }
}

fn distance(left: &u64x4, right: &u64x4) -> u64 {
    let diff = left.abs_diff(*right);
    (diff*diff).reduce_sum()
}

fn parse_input(input: &[u8]) -> Parser<'_> {
    Parser { lines: lines(input) }
}

fn read_int<'a>(line: &'a[u8], scan: &mut u64) -> (u64, &'a[u8]) {
    if *scan == 0 {
        return (u64::from_bytes(&line[..]), &[]);
    }

    let position = scan.trailing_zeros();
    *scan >>= position;
    *scan >>= scan.trailing_ones();

    (u64::from_bytes(&line[..position as usize]), &line[position as usize + 1..])
}

fn main() {
    let input = parse_input(include_bytes!("../../inputs/day8.txt"));

    println!("Part 1 Result: {}", part_one(input, 1000));
    println!("Part 2 Result: {}", part_two(input));
}

fn part_one(parser: Parser, limit: usize) -> usize {
    let (distance, _total) = calculate_initial_state(parser);

    let mut circuits = distance.iter().take(limit).fold(Vec::new(), connect_circuit);
    circuits.sort_by(|a, b| b.len().cmp(&a.len()));

    circuits.into_iter().take(3).fold(1, |acc, v| acc * v.len())
}

fn part_two(parser: Parser) -> u64 {
    let (distance, total) = calculate_initial_state(parser);

    let result = distance.iter().try_fold(
        Vec::new(),
        connect_circuit_until_circuit_length(total)
    );

    let value = result.break_value().expect("Must be set after iterations");
    value.0.0 * value.1.0
}

fn calculate_initial_state(parser: Parser) -> (HeapWithLimit, usize) {
    let (junction_boxes, distance) = parser.fold(
        (Vec::new(), HeapWithLimit::new()),
        |(mut vectors, v), line| {
            let v = vectors.iter().fold(v, |mut v, other| {
                v.add(line.distance(other), *other, line);
                v
            });
            vectors.push(line);
            (vectors, v)
        }
    );

    let junctions = junction_boxes.len();

    (distance, junctions)
}

fn connect_circuit(mut circuits: Vec<Vec<Coordinate>>, (left, right): (Coordinate, Coordinate)) -> Vec<Vec<Coordinate>> {
    let _ = connect_pair(&mut circuits, left, right);
    circuits
}

fn connect_circuit_until_circuit_length(limit: usize) -> impl FnMut(Vec<Vec<Coordinate>>, (Coordinate, Coordinate)) -> ControlFlow<(Coordinate, Coordinate), Vec<Vec<Coordinate>>> {
    move |mut circuits, (left, right)| {
        match connect_pair(&mut circuits, left, right) {
            Some(position) if circuits[position].len() == limit => ControlFlow::Break((left, right)),
            _ => ControlFlow::Continue(circuits),
        }
    }
}

fn connect_pair(circuits: &mut Vec<Vec<Coordinate>>, left: Coordinate, right: Coordinate) -> Option<usize> {
    let circuit = (
        circuits.iter().position(|v| v.contains(&left)),
        circuits.iter().position(|v| v.contains(&right)),
    );

    let position = match circuit {
        (Some(source), Some(dest)) if source != dest => {
            let remove = source.max(dest);
            let keep = source.min(dest);

            let mut source = circuits.remove(remove);
            circuits[keep].append(&mut source);
            Some(keep)
        },
        (Some(dest), None) => {
            circuits[dest].push(right);
            Some(dest)
        },
        (None, Some(dest)) => {
            circuits[dest].push(left);
            Some(dest)
        },
        (None, None) => {
            circuits.push(vec![left, right]);
            Some(circuits.len() - 1)
        }
        _ => None
    };
    position
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> Parser<'static> {
        parse_input(b"162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689")
    }

    #[test]
    fn calculates_simd_distance_between_two_vectors() {
        assert_eq!(
            distance(
                &u64x4::load_or_default(&[162,817,812]),
                &u64x4::load_or_default(&[425,690,689]),
            ),
            100427
        );
    }

    #[test]
    fn test_case_part1() {
        assert_eq!(
            part_one(test_data(), 10),
            40
        );
    }

    #[test]
    fn test_case_part2() {
        assert_eq!(
            part_two(test_data()),
            25272
        );
    }

    #[test]
    fn parses_numbers() {
        assert_eq!(
            test_data().collect::<Vec<_>>(),
            vec![
                Coordinate(162, 817, 812),
                Coordinate(57, 618, 57),
                Coordinate(906, 360, 560),
                Coordinate(592, 479, 940),
                Coordinate(352, 342, 300),
                Coordinate(466, 668, 158),
                Coordinate(542, 29, 236),
                Coordinate(431, 825, 988),
                Coordinate(739, 650, 466),
                Coordinate(52, 470, 668),
                Coordinate(216, 146, 977),
                Coordinate(819, 987, 18),
                Coordinate(117, 168, 530),
                Coordinate(805, 96, 715),
                Coordinate(346, 949, 466),
                Coordinate(970, 615, 88),
                Coordinate(941, 993, 340),
                Coordinate(862, 61, 35),
                Coordinate(984, 92, 344),
                Coordinate(425, 690, 689)
            ]
        );
    }
}
