#![feature(portable_simd)]

use std::collections::HashSet;
use std::ops::ControlFlow;
use std::simd::prelude::*;
use aoc2025::{lines, Lines, NumberExt};

const PATTERN: u8x32 = u8x32::splat(b',');

#[derive(Clone,Copy)]
struct Parser<'a> {
    lines: Lines<'a>,
}

struct HeapWithLimit {
    limit: usize,
    items: Vec<(u64, u64x4, u64x4)>,
    index: usize,
}

impl HeapWithLimit {
    pub fn new(limit: usize) -> Self {
        Self {
            items: Vec::with_capacity(limit),
            limit,
            index: 0,
        }
    }

    pub fn add(&mut self, distance: u64, left: u64x4, right: u64x4) {
        let mut value = (distance, left, right);

        match (self.items.iter().position(|v| v.0 > distance), self.limit > self.items.len()) {
            (Some(position), true)  => {
                self.items.insert(position, value);
            },
            (None, true) => {
                self.items.insert(0, value);
            }
            (Some(position), false) => {
                std::mem::swap(&mut self.items[position], &mut value);
                if position + 1 < self.items.len() {
                    self.items.pop();
                    self.items.insert(position + 1, value);
                }
            },
            _ => {}
        }
    }
}

impl Iterator for HeapWithLimit {
    type Item = (u64x4, u64x4);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.items.len() {
            return None;
        }

        let (_, left, right) = self.items[self.index];
        self.index += 1;
        Some((left, right))
    }
}

impl Iterator for Parser<'_> {
    type Item = u64x4;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(mut line) => {
                let mut scan = u8x32::load_or_default(&line[..]).simd_eq(PATTERN).to_bitmask();
                if scan.count_ones() != 2 {
                    return None;
                }
                let mut vector = [0u64; 4];

                (vector[0], line) = read_int(line, &mut scan);
                (vector[1], line) = read_int(line, &mut scan);
                (vector[2], _) = read_int(line, &mut scan);

                Some(u64x4::from_array(vector))
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
    println!("Part 2 Result: {}", part_two(input, 500000));
}

fn part_one(parser: Parser, limit: usize) -> usize {
    let (distance, _total) = calculate_initial_state(parser, limit);

    let mut circuits = distance.fold(Vec::new(), connect_circuit);
    circuits.sort_by(|a, b| b.len().cmp(&a.len()));

    circuits.into_iter().take(3).fold(1, |acc, v| acc * v.len())
}

fn part_two(parser: Parser, limit: usize) -> u64 {
    let (mut distance, total) = calculate_initial_state(parser, limit);


    let result = distance.try_fold(
        Vec::new(),
        connect_circuit_until_circuit_length(total)
    );

    let _test = distance.items.iter().take(20).fold(HashSet::new(), |mut acc, (_, left, right)| {
        acc.insert(left);
        acc.insert(right);
        acc
    });

    let value = result.break_value().expect("Must be set after iterations");
    value.0[0] * value.1[0]
}

fn calculate_initial_state(parser: Parser, limit: usize) -> (HeapWithLimit, usize) {
    let (junction_boxes, distance) = parser.fold(
        (Vec::new(), HeapWithLimit::new(limit)),
        |(mut vectors, v), line| {
            let v = vectors.iter().fold(v, |mut v, other| {
                v.add(distance(other, &line), *other, line);
                v
            });
            vectors.push(line);
            (vectors, v)
        }
    );

    let junctions = junction_boxes.len();

    (distance, junctions)
}

fn connect_circuit(mut circuits: Vec<Vec<u64x4>>, (left, right): (u64x4, u64x4)) -> Vec<Vec<u64x4>> {
    let _ = connect_pair(&mut circuits, left, right);
    circuits
}

fn connect_circuit_until_circuit_length(limit: usize) -> impl FnMut(Vec<Vec<u64x4>>, (u64x4, u64x4)) -> ControlFlow<(u64x4, u64x4), Vec<Vec<u64x4>>> {
    move |mut circuits, (left, right)| {
        match connect_pair(&mut circuits, left, right) {
            Some(position) if circuits[position].len() == limit => ControlFlow::Break((left, right)),
            _ => ControlFlow::Continue(circuits),
        }
    }
}

fn connect_pair(circuits: &mut Vec<Vec<u64x4>>, left: u64x4, right: u64x4) -> Option<usize> {
    let (_first_x, _second_x) = (left.to_array()[0], right.to_array()[0]);

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
        _other => None
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
            part_two(test_data(), 190),
            25272
        );
    }

    #[test]
    fn parses_numbers() {
        assert_eq!(
            test_data().collect::<Vec<_>>(),
            vec![
                u64x4::from_array([162, 817, 812, 0]),
                u64x4::from_array([57, 618, 57, 0]),
                u64x4::from_array([906, 360, 560, 0]),
                u64x4::from_array([592, 479, 940, 0]),
                u64x4::from_array([352, 342, 300, 0]),
                u64x4::from_array([466, 668, 158, 0]),
                u64x4::from_array([542, 29, 236, 0]),
                u64x4::from_array([431, 825, 988, 0]),
                u64x4::from_array([739, 650, 466, 0]),
                u64x4::from_array([52, 470, 668, 0]),
                u64x4::from_array([216, 146, 977, 0]),
                u64x4::from_array([819, 987, 18, 0]),
                u64x4::from_array([117, 168, 530, 0]),
                u64x4::from_array([805, 96, 715, 0]),
                u64x4::from_array([346, 949, 466, 0]),
                u64x4::from_array([970, 615, 88, 0]),
                u64x4::from_array([941, 993, 340, 0]),
                u64x4::from_array([862, 61, 35, 0]),
                u64x4::from_array([984, 92, 344, 0]),
                u64x4::from_array([425, 690, 689, 0])
            ]
        );
    }
}
