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

    fn is_complete(&self) -> bool {
        !self
            .tiles
            .iter()
            .any(|row| row.iter().any(|t| matches!(t, Tile::Cherry)))
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

    fn find_snake_head(&self) -> Option<Position> {
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

        if cfg!(debug_assertions) {
            dbg!(&self);
            eprintln!("Moving snake {dir:?}");
        }

        let (sx, sy) = self
            .find_snake_head()
            .expect("Can't find snake before it's placed");

        let (nx, ny) = match dir {
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
        }?;

        match self.tiles[ny][nx] {
            Rock => None,
            Cherry => {
                self.tiles[sy][sx] = SnakeBody;
                self.tiles[ny][nx] = SnakeHead;

                if let Some(b) = self.clone().move_snake(dir) {
                    b
                } else {
                    self
                }
                .into()
            }
            SnakeBody => None,
            SnakeHead => unreachable!(),
        }
    }

    fn moves(&self) -> impl Iterator<Item = Self> + '_ {
        use Dir::*;

        [Up, Down, Right, Left]
            .into_iter()
            .filter_map(|dir| self.clone().move_snake(dir))
    }
}

#[derive(Debug, Clone)]
enum BoardParent {
    Board(Board),
    SnakeStart(Position),
}

fn solution(board: Board, hash: HashMap<Board, BoardParent>) -> (Position, Vec<Dir>) {
    use BoardParent::*;
    use Dir::*;

    for (k, v) in &hash {
        dbg!(k.find_snake_head(),);

        match v {
            SnakeStart(p) => {
                dbg!(p);
            }
            Board(b) => {
                dbg!(b.find_snake_head());
            }
        }
    }

    let mut path: Vec<_> = successors(Some(Board(board)), {
        // eprintln!("Assembling path");
        |bp| match bp {
            Board(b) => hash.get(b).cloned(),
            SnakeStart(p) => Some(SnakeStart(*p)),
        }
    })
    // .inspect(|x| {
    //     dbg!(x);
    // })
    .collect();

    dbg!(&path);

    path.reverse();

    let pos = match path[0] {
        SnakeStart(p) => p,
        Board(_) => unreachable!("The first one should always be a position"),
    };

    let deltas = path
        .windows(2)
        .into_iter()
        .skip(1)
        .map(|window| match window {
            [Board(b1), Board(b2)] => match (b1.find_snake_head(), b2.find_snake_head()) {
                (Some((x1, y1)), Some((x2, y2))) => {
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
                _ => unreachable!("Every board in the history should have a snake head"),
            },
            _ => unimplemented!(
                "These should all be slices of BoardParent::Board with a length of 2"
            ),
        })
        .collect();

    (pos, deltas)
}

fn solve(input: &str) -> Option<(Position, Vec<Dir>)> {
    let board = Board::new(input);

    let solution = board.starting_positions().find_map(|p| {
        let mut visited = HashMap::from([(board.place_snake(p), BoardParent::SnakeStart(p))]);
        let mut queue = VecDeque::from([board.place_snake(p)]);

        while let Some(b) = queue.pop_front() {
            eprint!(".");

            if cfg!(debug_assertions) {
                dbg!(&b);
            }

            if b.is_complete() {
                return solution(b, visited).into();
            }

            for m in b.moves() {
                if !visited.contains_key(&m) {
                    visited.insert(m.clone(), BoardParent::Board(b.clone()));
                    queue.push_back(m);
                }
            }
        }

        None
    });

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
