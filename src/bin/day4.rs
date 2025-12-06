#![feature(portable_simd)]

use std::collections::VecDeque;
use std::simd::prelude::*;

fn main() {
    let input = include_bytes!("../../inputs/day4.txt");
    let mut grid: Vec<[_; 256]> = parse_grid(input);
    println!("Result Part1: {}", part1_available_rolls(&grid));
    println!("Result Part1: {}", part2_available_rolls(&mut grid));
}

fn parse_grid<const L: usize>(input: &[u8]) -> Vec<[i8; L]> {
    input.split(|&c| c == b'\n')
        .filter(|line| !line.is_empty())
        .map(slice_into_bitmask)
        .collect()
}

fn part1_available_rolls<const L: usize>(grid: &[[i8; L]], ) -> u64 {
    let mut move_list = grid.iter().enumerate()
        .fold(
            VecDeque::new(),
            |move_list, (row, line)| line.iter().enumerate()
                .fold(move_list, |mut move_list, (col, value)| {
                    move_list.push_back(Cell::Value(row, col));
                    move_list
                }));

    let mut total = 0;

    while let Some(cell) = move_list.pop_front() {
        if (cell.value(&grid) == -1) {
            if cell.neighbours(grid).reduce_sum() > -4 {
                total+=1;
            }
        }
    }

    total
}

fn part2_available_rolls<const L: usize>(grid: &mut [[i8; L]]) -> u64 {
    let mut move_list = grid.iter().enumerate()
        .fold(
            VecDeque::new(),
            |move_list, (row, line)| line.iter().enumerate()
                .fold(move_list, |mut move_list, (col, value)| {
                    move_list.push_back(Cell::Value(row, col));
                    move_list
                }));

    let mut total = 0;

    while let Some(cell) = move_list.pop_front() {
        if (cell.value(&grid) == -1) {
            if cell.neighbours(grid).reduce_sum() > -4 {
                total+=1;
                cell.remove(grid);
                cell.fill_non_empty_neighbours(grid, &mut move_list);
            }
        }
    }

    total
}

pub fn display_grid<const L: usize>(grid: &[[i8; L]], width: usize) -> String {
    grid.iter().enumerate().map(|(row, line)|
        line
            .iter().enumerate().take(width).map(move |(col, value)| {
            if *value == -1 {
                if neighbours(grid, (row, col)).reduce_sum() <= -4 {
                    '@'
                } else {
                    'x'
                }
            } else { '.' }
        }).chain(['\n'])).flatten().collect::<String>()
}

const ROLL_PATTERN: u8x8 = u8x8::splat(b'@');

fn slice_into_bitmask<const L: usize>(input: &[u8]) -> [i8; L] {
    let mut result = [0; L];
    if input.len() > L || L % 8 > 0 {
        return result;
    }

    let chunks = input.chunks_exact(8);
    let remainder = chunks.remainder();
    let mut index = 0;
    for chunk in chunks {
        let value = u8x8::load_or_default(chunk)
            .simd_eq(ROLL_PATTERN)
            .to_int()
            .to_array();

        assign_byte_slice(&mut result, index, value);
        index += 8;
    }

    let value = u8x8::load_or_default(remainder)
        .simd_eq(ROLL_PATTERN)
        .to_int()
        .to_array();

    assign_byte_slice(&mut result, index, value);
    result
}

fn assign_byte_slice(result: &mut [i8], index: usize, value: [i8; 8]) {
    result[index] = value[0];
    result[index + 1] = value[1];
    result[index + 2] = value[2];
    result[index + 3] = value[3];
    result[index + 4] = value[4];
    result[index + 5] = value[5];
    result[index + 6] = value[6];
    result[index + 7] = value[7];
}

#[derive(Copy, Clone)]
enum Cell {
    None,
    Value(usize, usize),
}

impl Cell {
    fn value<const L: usize>(&self, grid: &[[i8; L]]) -> i8 {
        match self {
            Cell::None => 0,
            Cell::Value(row, cell) => grid[*row][*cell]
        }
    }

    fn remove<const L: usize>(&self, grid: &mut [[i8; L]]) {
        match self {
            Cell::Value(row, cell) => {
                grid[*row][*cell] = 0;
            },
            _ => {}
        }
    }

    fn neighbours<const L: usize>(&self, grid: &[[i8; L]]) -> Simd<i8,8> {
        match self {
            Cell::Value(row, cell) => {
                let result = neighbour_cells(
                    grid.len(),
                    L,
                    *row,
                    *cell
                );

                result.map(|c| c.value(grid)).into()
            },
            _ => Default::default()
        }
    }

    fn fill_non_empty_neighbours<const L: usize>(&self, grid: &[[i8; L]], cells: &mut VecDeque<Cell>)  {
        match self {
            Cell::Value(row, cell) => {
                for cell in neighbour_cells(
                    grid.len(),
                    L,
                    *row,
                    *cell
                ) {
                    if (cell.value(grid) == -1) {
                        cells.push_back(cell);
                    }
                }
            },
            _ => Default::default()
        }
    }
}

fn neighbour_cells(max_rows: usize, max_cols: usize, row: usize, col: usize) -> [Cell; 8] {
    let mut result = [Cell::None; 8];
    if row > 0 {
        if col > 0 {
            result[0] = Cell::Value(row - 1, col - 1);
        }
        result[1] = Cell::Value(row - 1, col);
        if col < max_cols - 1 {
            result[2] = Cell::Value(row - 1, col + 1);
        }
    }
    if col > 0 {
        result[3] = Cell::Value(row, col - 1);
    }
    if col < max_cols - 1 {
        result[4] = Cell::Value(row, col + 1);
    }

    if row < max_rows - 1 {
        if col > 0 {
            result[5] = Cell::Value(row + 1, col - 1);
        }
        result[6] = Cell::Value(row + 1, col);
        if col < max_cols - 1 {
            result[7] = Cell::Value(row + 1, col + 1);
        }
    }

    result
}

fn neighbours<const L: usize>(grid: &[[i8; L]], (row, col): (usize, usize)) -> Simd<i8,8> {
    let result = neighbour_cells(
        grid.len(),
        L,
        row,
        col
    );

    result.map(|c| c.value(grid)).into()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grid() {
        assert_eq!(
            parse_grid(b"...@.\n..@..\n.....\n"),
            vec![
                [0, 0, 0, -1, 0, 0, 0, 0],
                [0, 0, -1, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn eight_cells() {
        let grid = [
            [0, 0, 0, -1, 0, 0, 0, 0],
            [0, 0, -1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ];

        assert_eq!(
            neighbours(&grid, (0, 3)),
            [0, 0, 0, 0, 0, -1, 0, 0].into()
        );
        assert_eq!(
            neighbours(&grid, (1, 2)),
            [0, 0, -1, 0, 0, 0, 0, 0].into()
        );
    }

    #[test]
    fn part1_count_rolls() {
        let grid: Vec<[_; 16]> = parse_grid(
    b"..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.
");

        assert_eq!(display_grid(&grid, 10), "..xx.xx@x.
x@@.@.@.@@
@@@@@.x.@@
@.@@@@..@.
x@.@@@@.@x
.@@@@@@@.@
.@.@.@.@@@
x.@@@.@@@@
.@@@@@@@@.
x.x.@@@.x.
");

        assert_eq!(part1_available_rolls(&grid), 13);
    }

    #[test]
    fn part2_count_rolls_after_move() {
        let mut grid: Vec<[_; 16]> = parse_grid(
            b"..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.
");

        assert_eq!(part2_available_rolls(&mut grid), 43);
    }
}