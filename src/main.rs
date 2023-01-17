use std::collections::{HashMap, VecDeque};
use std::io::{stdin, Read};
use std::iter::successors;

type Position = (usize, usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Tile {
    Rock,
    Cherry,
    SnakeBody,
    SnakeHead,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Board {
    tiles: Vec<Vec<Tile>>,
}

#[derive(Debug, Clone, Copy)]
enum Dir {
    Up,
    Down,
    Right,
    Left,
}

impl Board {
    fn new(input: &str) -> Self {
        let tiles = input
            .trim()
            .lines()
            .map(|line| {
                line.trim()
                    .chars()
                    .map(|c| match c {
                        'r' => Tile::Rock,
                        _ => Tile::Cherry,
                    })
                    .collect()
            })
            .collect();

        Self { tiles }
    }

    fn cherry_count(&self) -> usize {
        self.tiles
            .iter()
            .map(|row| row.iter().filter(|t| matches!(t, Tile::Cherry)).count())
            .sum()
    }

    fn is_complete(&self) -> bool {
        self.cherry_count() == 0
    }

    fn starting_positions(&self) -> impl Iterator<Item = Position> + Clone + '_ {
        self.tiles.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(x, t)| matches!(t, Tile::Cherry).then_some((x, y)))
        })
    }

    fn place_snake(&self, (x, y): Position) -> Self {
        let mut tiles = self.tiles.clone();

        tiles[y][x] = Tile::SnakeHead;

        Self { tiles }
    }

    fn get_snake_head(&self) -> Option<Position> {
        self.tiles.iter().enumerate().find_map(|(y, row)| {
            row.iter()
                .enumerate()
                .find_map(|(x, t)| matches!(t, Tile::SnakeHead).then_some(x))
                .map(|x| (x, y))
        })
    }

    /// Move snake. Panics if snake has not been placed.
    fn move_snake(mut self, dir: Dir) -> Option<Self> {
        use Dir::*;
        use Tile::*;

        let (sx, sy) = self
            .get_snake_head()
            .expect("Can't find snake before it's placed");

        if cfg!(debug_assertions) {
            eprintln!("Snake is currently at ({sx}, {sy}).");
            eprintln!("Snake wants to move {dir:?}.");
        }

        let new_pos = match dir {
            Up => sy.checked_sub(1).map(|y| (sx, y)),
            Down => {
                if sy + 1 >= self.tiles.len() {
                    None
                } else {
                    Some((sx, sy + 1))
                }
            }
            Right => {
                if sx + 1 >= self.tiles[sy].len() {
                    None
                } else {
                    Some((sx + 1, sy))
                }
            }
            Left => sx.checked_sub(1).map(|x| (x, sy)),
        };

        if new_pos.is_none() {
            if cfg!(debug_assertions) {
                eprintln!("Snake is at the wall. Snake remains at ({sx}, {sy}).");
            }

            return self.into();
        }

        let (nx, ny) = new_pos.unwrap();

        if cfg!(debug_assertions) {
            eprintln!("Snake is trying to move to ({nx}, {ny}).");
        }

        match self.tiles[ny][nx] {
            a @ (Rock | SnakeBody) => {
                if cfg!(debug_assertions) {
                    eprintln!("The way is blocked by {a:?}. Snake remains at ({sx}, {sy}).");
                }

                self.into()
            }

            Cherry => {
                if cfg!(debug_assertions) {
                    eprintln!("The way is clear. Snake proceeds.");
                }

                self.tiles[sy][sx] = SnakeBody;
                self.tiles[ny][nx] = SnakeHead;

                self.move_snake(dir)
            }

            SnakeHead => unreachable!(),
        }
    }

    fn moves(&self) -> impl Iterator<Item = Self> + '_ {
        use Dir::*;

        [Up, Down, Right, Left].into_iter().filter_map(|dir| {
            self.clone().move_snake(dir).map(|new_board| {
                if cfg!(debug_assertions) {
                    eprintln!("{} cherries left.", new_board.cherry_count());
                }

                new_board
            })
        })
    }
}

fn solution(board: Board, history: HashMap<Board, Option<Board>>) -> (Position, Vec<Dir>) {
    use Dir::*;

    let mut path: Vec<_> = successors(Some(&board), {
        |b| history.get(b).and_then(|bp| bp.as_ref())
    })
    .filter_map(|b| b.get_snake_head())
    .collect();

    path.reverse();

    let pos = path[0];

    let deltas = path
        .windows(2)
        .into_iter()
        .map(|window| match window {
            [(x1, y1), (x2, y2)] => {
                if x1 > x2 {
                    Left
                } else if x1 < x2 {
                    Right
                } else if y1 > y2 {
                    Up
                } else {
                    Down
                }
            }
            _ => {
                unreachable!("These should all be slices of BoardParent::Board with a length of 2")
            }
        })
        .collect();

    (pos, deltas)
}

fn solve(input: &str) -> Option<(Position, Vec<Dir>)> {
    let board = Board::new(input);

    let solution = board.starting_positions().find_map(|p| {
        let board = board.place_snake(p);

        eprintln!("Starting from {p:?}");

        let mut visited = HashMap::from([(board.clone(), None)]);
        let mut queue = VecDeque::from([board]);

        while let Some(b) = queue.pop_front() {
            eprint!(".");

            if cfg!(debug_assertions) {
                eprintln!();
                eprintln!("{} moves tried.", visited.len());
            }

            if b.is_complete() {
                return solution(b, visited).into();
            }

            for m in b.moves() {
                if !visited.contains_key(&m) {
                    visited.insert(m.clone(), b.clone().into());
                    queue.push_back(m);

                    if cfg!(debug_assertions) {
                        eprintln!("Added one to queue.");
                    }
                }
            }
        }

        None
    });

    eprintln!();

    solution
}

fn main() {
    let mut input = String::new();

    stdin()
        .read_to_string(&mut input)
        .expect("Couldn't read input");

    let solution = solve(&input);

    if let Some(((x, y), moves)) = solution {
        println!("Solution found in {} moves.", moves.len());
        println!("Place snake at {x}, {y}");

        for (i, d) in moves.into_iter().enumerate() {
            println!("{i:2}. {d:?}");
        }
    } else {
        println!("No solution found.");
    }
}
