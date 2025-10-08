use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Garden {
    pub maze: HashSet<(i64, i64)>,
    pub herb_types: u64,
    pub herbs: HashMap<(i64, i64), char>,
    pub start: (i64, i64),
    pub size: (usize, usize),
}

impl Garden {
    pub fn parse(data: &str) -> Self {
        let mut maze = HashSet::<(i64, i64)>::new();
        let mut herb_types = 0;
        let mut herbs = HashMap::<(i64, i64), char>::new();
        let mut start = (0, 0);

        let lines = data.lines().collect::<Vec<_>>();
        let size = (lines.len(), lines[0].len());

        for (row, line) in data.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let loc = (row as i64, col as i64);
                match ch {
                    '#' | '~' => {}
                    '.' => {
                        maze.insert(loc);
                        if row == 0 {
                            start = loc;
                        }
                    }
                    'A'..='Z' => {
                        maze.insert(loc);
                        herbs.insert(loc, ch);
                        herb_types |= 1 << (ch as u8 - b'A');
                    }
                    _ => panic!("invalid character '{ch}'"),
                }
            }
        }

        Self {
            maze,
            herbs,
            herb_types,
            start,
            size,
        }
    }

    pub fn neighbors(&self, (remaining, pos): &(u64, (i64, i64))) -> Vec<(u64, (i64, i64))> {
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .iter()
            .filter_map(|delta| {
                let next = (pos.0 + delta.0, pos.1 + delta.1);
                if self.maze.contains(&next) {
                    let next_remaining = if let Some(h) = self.herbs.get(&next) {
                        let bit = 1u64 << (*h as u8 - b'A');
                        remaining & !bit
                    } else {
                        *remaining
                    };
                    Some((next_remaining, next))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn all_neighbors(&self, pos: &(i64, i64)) -> Vec<(i64, i64)> {
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .iter()
            .filter_map(|delta| {
                let next = (pos.0 + delta.0, pos.1 + delta.1);
                if self.maze.contains(&next) {
                    Some(next)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn is_end(&self, (remaining, pos): &(u64, (i64, i64))) -> bool {
        *remaining == 0 && *pos == self.start
    }
}
