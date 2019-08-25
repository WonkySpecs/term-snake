use std::time::Duration;
use std::{thread, io};
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;
use termion::AsyncReader;
use termion::input::Keys;
use std::io::{Write, stdout, stdin, Read};
use std::collections::HashMap;
use rand::{thread_rng, Rng};

extern crate termion;

const FRAME_DELAY_MS: u64 = 100;
const STARTING_LENGTH: u16 = 5;

fn main() {
    let mut stdin = termion::async_stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut head_pos = (HEIGHT / 2, WIDTH / 2);
    let mut head_dir = MoveDir::Up;
    let mut score = 0;
    let mut snake_length = STARTING_LENGTH;
    let mut rng = rand::thread_rng();
    let pellet_pos = (rng.gen_range(1, WIDTH - 1) as u16,
                      rng.gen_range(1, HEIGHT - 1) as u16);
    let mut grid = starting_position(&pellet_pos);
    const INFO_LINE: u16 = HEIGHT as u16 + 1;
    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    loop {
        write!(stdout, "{}{}",
               termion::cursor::Goto(1, 1),
               termion::clear::All).unwrap();
        let buf = draw_buffer(grid);
        for j in 0..HEIGHT {
            write!(stdout, "{}{}",
                   termion::cursor::Goto(1, j as u16 + 1),
                   buf[j].join("")).unwrap();
        }
        write!(stdout, "{}{}{}wasd to move, q to quit",
               termion::cursor::Goto(1, INFO_LINE),
               score,
               termion::cursor::Goto(WIDTH as u16 / 3, INFO_LINE)).unwrap();

        if let Some(movement) = get_direction(head_dir, &mut stdin) {
            head_dir = movement;
            let next_head_pos = next_head_position(
                head_pos,
                movement,
                &mut stdin);
            match grid[next_head_pos.0][next_head_pos.1].obj_type {
                ObjType::Snake => break,
                ObjType::Food => {
                    score += snake_length;
                    snake_length += 1;
                    let pellet_pos = (rng.gen_range(1, WIDTH - 1) as u16,
                                      rng.gen_range(1, HEIGHT - 1) as u16);
                    grid[pellet_pos.1 as usize][pellet_pos.0 as usize] = GameObj::permanent(
                        ObjType::Food,
                        Option::None);
                },
                _ => ()
            };
            head_pos = next_head_pos;
        } else {
            break;
        }

        update_durations(&mut grid);

        grid[head_pos.0][head_pos.1] = GameObj {
            obj_type: ObjType::Snake,
            duration: Option::Some(snake_length),
            direction: Option::Some(SegmentDir::NS),
        };
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(FRAME_DELAY_MS));
    }

    write!(stdout, "{}{}Your final score was:{}{}Try (a)gain, or (q)uit?",
           termion::cursor::Goto(1, 1),
           termion::clear::All,
           score,
           termion::cursor::Goto(1, 2)).unwrap();
    stdout.flush().unwrap();
    let stdin = io::stdin();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => println!("Quitting..."),
            _ => println!("jk just run again")
        }
        break;
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

const WIDTH: usize = 40;

const HEIGHT: usize = 30;

#[derive(Copy, Clone)]
struct GameObj {
    obj_type: ObjType,
    duration: Option<u16>,
    direction: Option<SegmentDir>,
}

impl GameObj {
    fn permanent(obj_type: ObjType, direction: Option<SegmentDir>) -> GameObj {
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

    fn get_symbol(&self) -> &'static str {
        match self.obj_type {
            ObjType::Empty => ".",
            ObjType::Wall => get_wall_icon(self.direction),
            ObjType::Snake => "x",
            ObjType::Food => "O"
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
enum SegmentDir {
    NS,
    EW,
    NE,
    ES,
    SW,
    WN,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MoveDir {
    Up,
    Down,
    Left,
    Right,
}

fn draw_buffer(grid: [[GameObj; WIDTH]; HEIGHT]) -> [[&'static str; WIDTH]; HEIGHT] {
    let mut buf = [[""; WIDTH]; HEIGHT];
    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            buf[j][i] = grid[j][i].get_symbol();
        }
    }
    buf
}

fn get_wall_icon(dir: Option<SegmentDir>) -> &'static str {
    if let Some(dir) = dir {
        match dir {
            SegmentDir::NS => "┃",
            SegmentDir::EW => "━",
            SegmentDir::NE => "┗",
            SegmentDir::ES => "┏",
            SegmentDir::SW => "┓",
            SegmentDir::WN => "┛",
        }
    } else {
        panic!("Wall without direction");
    }
}

fn starting_position(starting_pellet: &(u16, u16)) -> [[GameObj; WIDTH]; HEIGHT] {
    let mut grid: [[GameObj; WIDTH]; HEIGHT] =
        [[GameObj::blank(); WIDTH]; HEIGHT];
    let h_wall = GameObj::permanent(
        ObjType::Wall,
        Option::Some(SegmentDir::EW));

    for i in 0..WIDTH {
        grid[0][i] = h_wall.clone();
        grid[HEIGHT - 1][i] = h_wall.clone();
    }

    let v_wall = GameObj::permanent(
        ObjType::Wall,
        Option::Some(SegmentDir::NS));

    for j in 0..HEIGHT {
        grid[j][0] = v_wall.clone();
        grid[j][WIDTH - 1] = v_wall.clone();
    }

    let mut corner_dirs = HashMap::new();
    corner_dirs.insert((0, 0), SegmentDir::ES);
    corner_dirs.insert((0, WIDTH - 1), SegmentDir::SW);
    corner_dirs.insert((HEIGHT - 1, WIDTH - 1), SegmentDir::WN);
    corner_dirs.insert((HEIGHT - 1, 0), SegmentDir::NE);

    for ((i, j), dir) in corner_dirs.into_iter() {
        grid[i][j] = GameObj::permanent(
            ObjType::Wall,
            Option::Some(dir));
    }

    grid[HEIGHT / 2][WIDTH / 2] = GameObj {
        obj_type: ObjType::Snake,
        direction: Option::Some(SegmentDir::NS),
        duration: Option::Some(STARTING_LENGTH),
    };

    grid[starting_pellet.1 as usize][starting_pellet.0 as usize] = GameObj::permanent(
        ObjType::Food,
        Option::None);

    grid
}

fn get_direction(cur_dir: MoveDir,
                 input_reader: &mut AsyncReader) -> Option<MoveDir> {
    let mut inputs: Vec<u8> = Vec::new();
    input_reader.read_to_end(&mut inputs).unwrap();
    if let Some(last_input) = inputs.last() {
        Option::Some(match last_input {
            b'q' => return Option::None,
            b'w' => MoveDir::Up,
            b's' => MoveDir::Down,
            b'a' => MoveDir::Left,
            b'd' => MoveDir::Right,
            _ => cur_dir
        })
    } else {
        Option::Some(cur_dir)
    }
}

fn next_head_position(cur_pos: (usize, usize),
                      cur_dir: MoveDir,
                      input_reader: &mut AsyncReader) -> (usize, usize) {
    let head_movement: (i8, i8) = match cur_dir {
        MoveDir::Left => (0, -1),
        MoveDir::Right => (0, 1),
        MoveDir::Up => (-1, 0),
        MoveDir::Down => (1, 0),
    };

    let mut new_head_pos =
        (cur_pos.0 as i8 + head_movement.0,
         cur_pos.1 as i8 + head_movement.1);
    // There's probably a nice way of doing this with modular arithmetic but cba
    if new_head_pos.0 < 1 {
        new_head_pos.0 = HEIGHT as i8 - 2;
    } else if new_head_pos.0 > HEIGHT as i8 - 2 {
        new_head_pos.0 = 1;
    }

    if new_head_pos.1 < 1 {
        new_head_pos.1 = WIDTH as i8 - 2;
    } else if new_head_pos.1 > WIDTH as i8 - 2 {
        new_head_pos.1 = 1;
    }

    (new_head_pos.0 as usize, new_head_pos.1 as usize)
}

fn update_durations(grid: &mut [[GameObj; WIDTH]; HEIGHT]) {
    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            if let Some(dur) = grid[j][i].duration {
                let next_dur = dur - 1;
                if next_dur < 1 {
                    grid[j][i] = GameObj::blank();
                } else {
                    grid[j][i].duration = Option::Some(next_dur);
                }
            }
        }
    }
}