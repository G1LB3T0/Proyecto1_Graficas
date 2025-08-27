// maze.rs

use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.unwrap().chars().collect())
        .collect()
}

pub fn load_maze_for_level(level: i32) -> Maze {
    let filename = match level {
        1 => "maze1.txt",
        2 => "maze2.txt",
        3 => "maze3.txt",
        _ => "maze1.txt", // fallback
    };
    load_maze(filename)
}
