#![allow(unused_variables)]
use std::env;
use std::fs;
use std::io::stdin;
use std::io::Read;
use std::process::exit;
use std::str::from_utf8;
use std::u16;

#[derive(Debug, PartialEq)]
enum Instruction {
    // (op_num)
    MoveLeft(usize),
    MoveRight(usize),
    Increment(usize),
    Decrement(usize),
    Input(usize),
    Output(usize),
    // (op_num, loop_count)
    Open(usize, usize),
    Close(usize, usize),
}

impl Instruction {
    pub fn decode(op: char, loop_count: usize, pos: usize) -> Self {
        use Instruction::*;
        match op {
            '<' => MoveLeft(pos),
            '>' => MoveRight(pos),
            '+' => Increment(pos),
            '-' => Decrement(pos),
            '.' => Output(pos),
            ',' => Input(pos),
            '[' => Open(pos, loop_count),
            ']' => Close(pos, loop_count),
            _ => unreachable!(),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let args: Vec<String> = args[1..].to_vec();
    let mut instructions: Vec<Instruction> = vec![];

    if args.is_empty() {
        eprintln!("Missing command line arguments")
    } else {
        for arg in &args {
            println!("{}", *arg);
            if *arg == "-c" {
                let src: String = args[1..].join("");
                println!("Parsing: {}", src);
                instructions = parse(&src);

                println!("{:#?}", instructions);

                break;
            } else {
                // attempt to open file
                let mut file = match fs::File::open(arg) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Could not open file: {}", e);

                        exit(-1)
                    }
                };

                let mut buff = String::new();

                if let Err(e) = file.read_to_string(&mut buff) {
                    eprintln!("Could not read file: {}", e)
                }

                println!("Parsing: {}", buff);
                instructions = parse(&buff);
            }
        }

        execute(&instructions);
    }
}

fn parse(src: &str) -> Vec<Instruction> {
    let ops: Vec<char> = src
        .chars()
        .filter(|c| match *c {
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
            _ => false,
        })
        .collect();

    let mut instructions = vec![];

    // loop ID
    let mut loop_count = 0;
    for (op_num, op) in ops.iter().enumerate() {
        match *op {
            '[' => {
                instructions.push(Instruction::decode(*op, loop_count, op_num));
                loop_count += 1;
            }
            ']' => {
                assert!(loop_count > 0, "Unmatched brace at: {}", op_num);
                loop_count -= 1;

                instructions.push(Instruction::decode(*op, loop_count, op_num));
            }
            _ => {
                //println!("Pushing: {}", *op);
                instructions.push(Instruction::decode(*op, 0, op_num));
            }
        }
    }

    if loop_count > 0 {
        panic!("Unmatched brace at EOF")
    }

    instructions
}

fn execute(instructions: &[Instruction]) {
    let mut mem: [u8; u16::MAX as usize] = [0; u16::MAX as usize];
    let mut data_pointer: usize = 0;
    let mut stack: Vec<usize> = vec![];

    //println!("{:#?}", src);

    let mut bookmark: usize = 0;
    'main: loop {
        'execution: for (pos, instruction) in instructions
            .iter()
            .skip(stack.pop().unwrap_or_else(|| bookmark))
            .enumerate()
        {
            //println!("{:?}", instruction);
            match *instruction {
                Instruction::MoveRight(_) => {
                    data_pointer += 1;
                    eprintln!("Moving right");
                }
                Instruction::MoveLeft(_) => {
                    data_pointer -= 1;
                    eprintln!("Moving left");
                }
                Instruction::Increment(_) => {
                    mem[data_pointer] += 1;
                    eprintln!("Incrementing");
                }
                Instruction::Decrement(_) => {
                    mem[data_pointer] -= 1;
                    eprintln!("Decrementing");
                }
                Instruction::Output(_) => println!("{}", char::from(mem[data_pointer])),
                Instruction::Input(_) => {
                    mem[data_pointer] = stdin().bytes().next().unwrap().unwrap()
                }
                Instruction::Open(pos, _) => {
                    if mem[data_pointer] != 0 {
                        stack.push(pos + 1);
                    } else {
                        bookmark = match instructions.iter().position(|op| match (op, instruction) {
                            (
                                Instruction::Close(close_pos, close_loop_count),
                                Instruction::Open(open_pos, open_loop_count),
                            ) => *close_loop_count == *open_loop_count,
                            (_, _) => false,
                        }) {
                            Some(close_pos) => close_pos,
                            _ => unreachable!(),
                        };
                    }
                }
                Instruction::Close(pos, _) => {
                    if mem[data_pointer] != 0 {
                        continue 'main;
                    }
                }
            }
        }
        break 'main;
    }
}
