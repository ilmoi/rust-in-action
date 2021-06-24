use std::{env, thread};

use rayon::prelude::*;
use svg::Document;
use svg::node::element::{Path, Rectangle};
use svg::node::element::path::{Command, Position, Data};
use crossbeam::channel::unbounded;

use crate::Operation::{Forward, Home, Noop, TurnLeft, TurnRight};
use crate::Orientation::{East, North, South, West};

const WIDTH: isize = 400;
const HEIGHT: isize = WIDTH;
const HOME_Y: isize = WIDTH / 2;
const HOME_X: isize = WIDTH / 2;
const STROKE_WIDTH: usize = 5;

#[derive(Debug, Clone, Copy)]
enum Orientation {
    North,
    East,
    West,
    South, // <4> Using descriptions, rather than numerical values, avoids mathematics.
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    // <5> To produce richer output, feel free to extend the operations available to your programs.
    Forward(isize),
    // <6> Using an `isize` allows you to extend this example to implement a "Reverse" operation without adding a new variant.
    TurnLeft,
    TurnRight,
    Home,
    Noop(u8), // <7> Noop is used when we encounter illegal input. To write error messages, we retain the illegal byte.
}

#[derive(Debug)]
struct Artist {
    // <8> The `Artist` struct maintains the current state.
    x: isize,
    y: isize,
    heading: Orientation,
}

impl Artist {
    fn new() -> Artist {
        Artist {
            heading: North,
            x: HOME_X,
            y: HOME_Y,
        }
    }
    fn home(&mut self) {
        self.x = HOME_X;
        self.y = HOME_Y;
    }

    fn forward(&mut self, distance: isize) { // <9>
        match self.heading {                 // <9>
            North => self.y += distance,     // <9> `forward()` mutates `self` within the match expression.
            South => self.y -= distance,     // <9>
            West => self.x += distance,     // <9>
            East => self.x -= distance,     // <9>
        }                                    // <9>
    }                                        // <9>

    fn turn_right(&mut self) {              // <10>
        self.heading = match self.heading { // <10> `turn_left()` and `turn_right()` mutate `self` outside of the match expression.
            North => East,                  // <10>
            South => West,                  // <10>
            West => North,                 // <10>
            East => South,                 // <10>
        }                                   // <10>
    }

    fn turn_left(&mut self) {               // <10>
        self.heading = match self.heading { // <10>
            North => West,                  // <10>
            South => East,                  // <10>
            West => South,                 // <10>
            East => North,                 // <10>
        }                                   // <10>
    }                                       // <10>

    fn wrap(&mut self) {  // <11> `wrap()` ensures that the drawing stays within bounds.
        if self.x < 0 {
            self.x = HOME_X;
            self.heading = West;
        } else if self.x > WIDTH {
            self.x = HOME_X;
            self.heading = East;
        }

        if self.y < 0 {
            self.y = HOME_Y;
            self.heading = North;
        } else if self.y > HEIGHT {
            self.y = HOME_Y;
            self.heading = South;
        }
    }
}

// todo single thread
///❯ echo "hello world" | sha256sum | cut -f1 -d' '
/// a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447
/// returns a vector of Operations (our own enum)
// fn parse(input: &str) -> Vec<Operation> {
//     let mut steps = Vec::<Operation>::new();
//     // 97 -> 57 -> etc...
//     for byte in input.bytes() {
//         let step = match byte {
//             b'0' => Home, //byte 48
//             b'1'..=b'2' => { //bytes 49 - 57
//                 // subtract 48 in decimal - coz 1-9 in ascii start from 49 https://design215.com/toolbox/ascii-utf8.php#:~:text=UTF%2D8%20is%20variable%20width,both%20modern%20and%20ancient%20languages.
//                 let distance = (byte - 0x30) as isize;
//                 Forward(distance * (HEIGHT / 10))
//             },
//             //reverse - just use a negative sign
//             b'3'..=b'9' => {
//                 let distance = (byte - 0x30) as isize;
//                 Forward(-distance * (HEIGHT / 10))
//             }
//             b'a' | b'b' | b'c' => TurnLeft, // bytes 97-99
//             b'd' | b'e' | b'f' => TurnRight, // bytes 100-102
//             _ => Noop(byte)
//         };
//         steps.push(step);
//     }
//     steps
// }

// todo multi thread - using rayon
// fn parse(input: &str) -> Vec<Operation> {
//     // 97 -> 57 -> etc...
//     // par_iter method automatically turns on safe multi-threading
//     input.as_bytes().par_iter().map(|byte| {
//         match byte {
//             b'0' => Home, //byte 48
//             b'1'..=b'2' => { //bytes 49 - 57
//                 // subtract 48 in decimal - coz 1-9 in ascii start from 49 https://design215.com/toolbox/ascii-utf8.php#:~:text=UTF%2D8%20is%20variable%20width,both%20modern%20and%20ancient%20languages.
//                 let distance = (byte - 0x30) as isize;
//                 Forward(distance * (HEIGHT / 10))
//             }
//             //reverse - just use a negative sign
//             b'3'..=b'9' => {
//                 let distance = (byte - 0x30) as isize;
//                 Forward(-distance * (HEIGHT / 10))
//             }
//             b'a' | b'b' | b'c' => TurnLeft, // bytes 97-99
//             b'd' | b'e' | b'f' => TurnRight, // bytes 100-102
//             _ => Noop(*byte)
//         }
//     })
//         .collect()
// }

enum Work {
    Task((usize, u8)),
    Finished
}

// todo channel approach - using crossbeam
fn parse(input: &str) -> Vec<Operation> {
    let n_threads = 2;
    let (todo_tx, todo_rx) = unbounded();
    let (results_tx, results_rx) = unbounded();

    let mut n_bytes = 0;

    for (i, byte) in input.bytes().enumerate() {
        todo_tx.send(Work::Task((i, byte))).unwrap();
        n_bytes += 1;
    }

    for _ in 0..n_threads {
        todo_tx.send(Work::Finished).unwrap();
    }

    for _ in 0..n_threads {
        let todo = todo_rx.clone();
        let results = results_tx.clone();
        thread::spawn(move || {
            loop {
                let task = todo.recv();
                let result = match task {
                    Err(_) => break,
                    Ok(Work::Finished) => break,
                    Ok(Work::Task((i, byte))) => (i, parse_byte(byte)),
                };
                results.send(result).unwrap();
            }
        });
    }

    // create an empty vector, coz we'll be populating it in random order depending on how threads return
    let mut ops = vec![Noop(0); n_bytes];
    for _ in 0..n_bytes {
        let (i, op) = results_rx.recv().unwrap();
        ops[i] = op;
    }

    ops


}

fn parse_byte(byte: u8) -> Operation {
    match byte {
        b'0' => Home, //byte 48
        b'1'..=b'2' => { //bytes 49 - 57
            // subtract 48 in decimal - coz 1-9 in ascii start from 49 https://design215.com/toolbox/ascii-utf8.php#:~:text=UTF%2D8%20is%20variable%20width,both%20modern%20and%20ancient%20languages.
            let distance = (byte - 0x30) as isize;
            Forward(distance * (HEIGHT / 10))
        }
        //reverse - just use a negative sign
        b'3'..=b'9' => {
            let distance = (byte - 0x30) as isize;
            Forward(-distance * (HEIGHT / 10))
        }
        b'a' | b'b' | b'c' => TurnLeft, // bytes 97-99
        b'd' | b'e' | b'f' => TurnRight, // bytes 100-102
        _ => Noop(byte)
    }
}

fn convert(operations: &Vec<Operation>) -> Vec<Command> {
    //spawn a new turtle
    let mut turtle = Artist::new();

    //create a vector that will hold the path data
    let mut path_data = Vec::<Command>::with_capacity(1 + operations.len());

    //push the starting position into that vector
    path_data.push(Command::Move(Position::Absolute, (HOME_X, HOME_Y).into()));

    //parse the operations vec we prepared in prev function, using the methods we defined on turtle
    for op in operations {
        match *op {
            Forward(distance) => turtle.forward(distance),
            TurnLeft => turtle.turn_left(),
            TurnRight => turtle.turn_right(),
            Home => turtle.home(),
            Noop(byte) => eprintln!("warning: illegal byte encountered: {:?}", byte),
        };
        //this is where we build the Line element before pushing it into the vec
        path_data.push(Command::Line(Position::Absolute, (turtle.x, turtle.y).into()));
        turtle.wrap();
    }

    println!("Path data is: {:?}", path_data);
    path_data
}

fn generate_svg(path_data: Vec<Command>) -> Document {
    let background = Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", WIDTH)
        .set("height", HEIGHT)
        .set("fill", "#ffffff");

    let border = background.clone()
        .set("fill-opacity", "0.0")
        .set("stroke", "#cccccc")
        .set("stroke-width", 3 * STROKE_WIDTH);

    let sketch = Path::new()
        .set("fill", "none")
        .set("stroke", "#2f2f2f")
        .set("stroke-width", STROKE_WIDTH)
        .set("stroke-opacity", "0.9")
        .set("d", Data::from(path_data));

    let document = Document::new()
        .set("viewBox", (0, 0, HEIGHT, WIDTH))
        .set("height", HEIGHT)
        .set("width", WIDTH)
        .set("style", "style=\"outline: 5px solid #800000;\"")
        .add(background)
        .add(sketch)
        .add(border);

    document
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let input = args.get(1).unwrap();
    let default_filename = format!("{}.svg", input);
    let save_to = args.get(2).unwrap_or(&default_filename);

    let operations = parse(input);
    let path_data = convert(&operations);
    let document = generate_svg(path_data);
    svg::save(save_to, &document).unwrap();
}