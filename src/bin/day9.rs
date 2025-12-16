#![feature(portable_simd)]
use std::simd::prelude::*;
use aoc2025::{lines, measure, Lines, NumberExt, Task};

fn main() {
    let input = include_bytes!("../../inputs/day9.txt");

    measure(Task::Part1, || part1(Parser::new(input)));
}

fn part1(input: Parser) -> u64 {
    let best_box = input.fold(BoundingBox::default(), |acc, point| {
        acc.adjust(point)
    }).best_box();

    best_box.1.area(best_box.0)
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

#[derive(Clone,Copy,PartialEq,Eq,Debug,Default)]
struct Coordinate(u32, u32);

impl Coordinate {
    fn distance(&self, target: Coordinate) -> u32 {
        let dx = target.0.abs_diff(self.0);
        let dy = target.1.abs_diff(self.1);

        (dx * dx + dy * dy).isqrt()
    }

    fn area(self, other: Coordinate) -> u64 {
        let dx = other.0.abs_diff(self.0) as u64 + 1;
        let dy = other.1.abs_diff(self.1) as u64 + 1;

        dx * dy
    }

    fn closest(self, other: &Self, reference: Self) -> Self {
        if self.distance(reference) > other.distance(reference) {
            return *other
        }

        self
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Debug,Default)]
struct BoundingBox {
    edges: Option<(u32, u32, u32, u32)>,
    by_distance: Option<(Coordinate, Coordinate, Coordinate, Coordinate)>
}

impl BoundingBox {
    fn adjust(self, other: Coordinate) -> Self {
        match (self.edges, self.by_distance) {
            (None, None) => Self {
                edges: Some((other.0, other.0, other.1, other.1)),
                by_distance: Some((other, other, other, other))
            },
            (
                Some((min_x, max_x, min_y, max_y)),
                Some((top_left, top_right, bottom_left, bottom_right))
            ) => {
                let (min_x, max_x, min_y, max_y) = (
                    min_x.min(other.0),
                    max_x.max(other.0),
                    min_y.min(other.1),
                    max_y.max(other.1)
                );

                let (top_left, top_right, bottom_left, bottom_right) = (
                    top_left.closest(&other, Coordinate(min_x, min_y)),
                    top_right.closest(&other, Coordinate(max_x, min_y)),
                    bottom_left.closest(&other, Coordinate(min_x, max_y)),
                    bottom_right.closest(&other, Coordinate(max_x, max_y)),
                );

                Self {
                    edges: Some((min_x, max_x, min_y, max_y)),
                    by_distance: Some((top_left, top_right, bottom_left, bottom_right)),
                }
            },
            _ => self
        }
    }

    fn best_box(&self) -> (Coordinate, Coordinate) {
        let (top_left, top_right, bottom_left, bottom_right) = self.by_distance.expect("no corners detected");

        if top_left.distance(bottom_right) > top_right.distance(bottom_left) {
            return (top_left, bottom_right)
        }

        (top_right, bottom_left)
    }
}

const PATTERN: u8x16 = u8x16::splat(b',');

impl Iterator for Parser<'_> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(line) => {
                let scan = u8x16::load_or_default(&line[..]).simd_eq(PATTERN).to_bitmask();
                if scan == 0 {
                    return None;
                }
                let position = scan.trailing_zeros() as usize;
                Some(Coordinate(
                    u32::from_bytes(&line[..position]),
                    u32::from_bytes(&line[position+1..])
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
    fn corner_finder() {
        let bounding_box = test_data().fold(BoundingBox::default(), |acc, point| {
            acc.adjust(point)
        });
        assert_eq!(
            bounding_box.by_distance,
            Some((Coordinate(2, 3), Coordinate(11, 1), Coordinate(2, 5), Coordinate(11, 7)))
        );
    }

    #[test]
    fn best_box() {
        let bounding_box = test_data().fold(BoundingBox::default(), |acc, point| {
            acc.adjust(point)
        });
        assert_eq!(
            bounding_box.best_box(),
            (Coordinate(11,1), Coordinate(2,5))
        );
    }

    #[test]
    fn test_case_part1() {
        assert_eq!(
            part1(test_data()),
            50
        );
    }
}
