use std::time::Duration;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
use std::iter::Map;
use std::collections::HashMap;

extern crate termion;

const FRAME_DELAY_MS: u64 = 200;
const WIDTH: usize = 40;
const HEIGHT: usize = 30;

#[derive(Copy, Clone)]
struct GameObj {
    obj_type: ObjType,
    duration: Option<u16>,
    direction: Option<Direction>,
}

impl GameObj {
    fn permanent(obj_type: ObjType, direction: Option<Direction>) -> GameObj {
        GameObj {
            obj_type,
            direction,
            duration: Option::None,
        }
    }

    fn blank() -> GameObj {
        GameObj {
            obj_type: ObjType::Empty,
            duration: Option::None,
            direction: Option::None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ObjType {
    Wall,
    Snake,
    Food,
    Empty,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Direction {
    NS,
    EW,
    NE,
    ES,
    SW,
    WN,
}

fn draw_buffer(grid: [[GameObj; WIDTH]; HEIGHT]) -> String {
    let mut s = String::from("");
    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            s += match grid[j][i].obj_type {
                ObjType::Empty => ".",
                ObjType::Wall => get_wall_icon(grid[j][i].direction),
                _ => "x"
            };
        }
        s += "\r\n";
    }
    s
}

fn get_wall_icon(dir: Option<Direction>) -> &'static str {
    if let Some(dir) = dir {
        match dir {
            Direction::NS => "┃",
            Direction::EW => "━",
            Direction::NE => "┗",
            Direction::ES => "┏",
            Direction::SW => "┓",
            Direction::WN => "┛",
        }
    } else {
        panic!("Wall without direction");
    }
}

fn main() {
    let mut playing = true;
    let stdin = stdin();
    // Get the standard output stream and go to raw mode.
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut grid = starting_position();
    let mut head_pos = (HEIGHT / 2, WIDTH / 2);

    write!(stdout, "{}{}q to exit. Type stuff, use alt, and so on.{}",
           // Clear the screen.
           termion::clear::All,
           // Goto (1,1).
           termion::cursor::Goto(1, 1),
           // Hide the cursor.
           termion::cursor::Hide).unwrap();
    // Flush stdout (i.e. make the output appear).
    stdout.flush().unwrap();

    let mut c = 0;
    while playing {
        write!(stdout, "{}{}{}",
               termion::cursor::Goto(1, 1),
               termion::clear::All,
               draw_buffer(grid)).unwrap();
        thread::sleep(Duration::from_millis(FRAME_DELAY_MS));
        c += 1;
        if c > 20 {
            playing = false;
        }
        head_pos.0 -= 1;
        grid[head_pos.0][head_pos.1] = GameObj {
            obj_type: ObjType::Snake,
            duration: Option::Some(1),
            direction: Option:: Some(Direction::NS)
        }
    }
}

fn starting_position() -> [[GameObj; WIDTH]; HEIGHT] {
    let mut grid: [[GameObj; WIDTH]; HEIGHT] =
        [[GameObj::blank(); WIDTH]; HEIGHT];
    let h_wall = GameObj::permanent(
        ObjType::Wall,
        Option::Some(Direction::EW));

    for i in 0..WIDTH {
        grid[0][i] = h_wall.clone();
        grid[HEIGHT - 1][i] = h_wall.clone();
    }

    let v_wall = GameObj::permanent(
        ObjType::Wall,
        Option::Some(Direction::NS));

    for j in 0..HEIGHT {
        grid[j][0] = v_wall.clone();
        grid[j][WIDTH - 1] = v_wall.clone();
    }

    let mut corner_dirs = HashMap::new();
    corner_dirs.insert((0, 0), Direction::ES);
    corner_dirs.insert((0, WIDTH - 1), Direction::SW);
    corner_dirs.insert((HEIGHT - 1, WIDTH - 1), Direction::WN);
    corner_dirs.insert((HEIGHT - 1, 0), Direction::NE);

    for ((i, j), dir) in corner_dirs.into_iter() {
        grid[i][j] = GameObj::permanent(
            ObjType::Wall,
            Option::Some(dir));
    }

    grid[HEIGHT / 2][WIDTH / 2] = GameObj {
        obj_type: ObjType::Snake,
        direction: Option::Some(Direction::NS),
        duration: Option::None,
    };

    grid
}