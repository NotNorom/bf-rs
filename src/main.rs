use std::{
    convert::TryFrom,
    io::{stdin, stdout, Read, Write},
    process::exit,
    str::FromStr,
    usize,
};

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
enum Instruction {
    IncrementDataPtr,
    DecrementDataPtr,
    Increment,
    Decrement,
    Output,
    Input,
    JmpForwardIfZero,
    JmpBackwardIfNonZero,
}

impl TryFrom<char> for Instruction {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '>' => Ok(Self::IncrementDataPtr),
            '<' => Ok(Self::DecrementDataPtr),
            '+' => Ok(Self::Increment),
            '-' => Ok(Self::Decrement),
            '.' => Ok(Self::Output),
            ',' => Ok(Self::Input),
            '[' => Ok(Self::JmpForwardIfZero),
            ']' => Ok(Self::JmpBackwardIfNonZero),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct Program {
    data: [u8; DATA_SIZE],
    data_pointer: usize,
    instructions: Vec<Instruction>,
    instruction_pointer: usize,
}

impl FromStr for Program {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let instructions: Vec<_> = s
            .chars()
            .map(Instruction::try_from)
            .filter_map(Result::ok)
            .collect();
        let p = Program {
            data: [0; DATA_SIZE],
            data_pointer: 0,
            instructions,
            instruction_pointer: 0,
        };

        Ok(p)
    }
}

impl Program {
    pub fn run(&mut self) -> Result<(), String> {
        loop {
            match self.instructions.get_mut(self.instruction_pointer) {
                Some(Instruction::IncrementDataPtr) => {
                    self.data_pointer += 1;
                    self.instruction_pointer += 1;
                }
                Some(Instruction::DecrementDataPtr) => {
                    self.data_pointer -= 1;
                    self.instruction_pointer += 1;
                }
                Some(Instruction::Increment) => {
                    self.data[self.data_pointer] = self.data[self.data_pointer].wrapping_add(1);
                    self.instruction_pointer += 1;
                }
                Some(Instruction::Decrement) => {
                    self.data[self.data_pointer] = self.data[self.data_pointer].wrapping_sub(1);
                    self.instruction_pointer += 1;
                }
                Some(Instruction::Output) => {
                    let content = self.data[self.data_pointer];
                    let _ = stdout().write_all(&[content]);
                    let _ = stdout().flush();
                    self.instruction_pointer += 1;
                }
                Some(Instruction::Input) => {
                    let maybe_content = stdin().bytes().next();
                    if let None = maybe_content {
                        self.instruction_pointer += 1;
                        continue;
                    }

                    self.data[self.data_pointer] =
                        maybe_content.unwrap().map_err(|e| e.to_string())?;
                    self.instruction_pointer += 1;
                }
                Some(Instruction::JmpForwardIfZero) => {
                    if self.data[self.data_pointer] != 0 {
                        self.instruction_pointer += 1;
                        continue;
                    }

                    let mut jmp_counter: usize = 1;
                    // backup ip for debug purpose
                    let old_ip = self.instruction_pointer;

                    while jmp_counter != 0 {
                        self.instruction_pointer += 1;
                        match self.instructions.get(self.instruction_pointer) {
                            Some(Instruction::JmpForwardIfZero) => jmp_counter += 1,
                            Some(Instruction::JmpBackwardIfNonZero) => jmp_counter -= 1,
                            None => {
                                return Err(format!(
                                    "Unmatched `[` instruction at position {}",
                                    old_ip
                                ))
                            }
                            _ => {}
                        }
                    }
                    self.instruction_pointer += 1;
                }
                Some(Instruction::JmpBackwardIfNonZero) => {
                    if self.data[self.data_pointer] == 0 {
                        self.instruction_pointer += 1;
                        continue;
                    }

                    let mut jmp_counter: usize = 1;
                    // backup ip for debug purpose
                    let old_ip = self.instruction_pointer;

                    while jmp_counter != 0 {
                        self.instruction_pointer -= 1;
                        match self.instructions.get(self.instruction_pointer) {
                            Some(Instruction::JmpForwardIfZero) => jmp_counter -= 1,
                            Some(Instruction::JmpBackwardIfNonZero) => jmp_counter += 1,
                            None => {
                                return Err(format!(
                                    "Unmatched `]` instruction at position {}",
                                    old_ip
                                ))
                            }
                            _ => {}
                        }
                    }
                    self.instruction_pointer += 1;
                }
                None => {
                    break;
                }
            }
        }
        Ok(())
    }
}

const DATA_SIZE: usize = 30_000;

fn main() {
    let mut args = std::env::args();
    let maybe_path = args.nth(1);
    if let None = maybe_path {
        eprintln!("Need file to interpret.\nPlease specify the path to your brainfuck file.");
        exit(-1);
    }
    let path = maybe_path.unwrap();

    let maybe_contents = std::fs::read_to_string(&path);
    if let Err(e) = maybe_contents {
        eprintln!("{}: {}", e.to_string(), &path);
        exit(-1);
    }
    let contents = maybe_contents.unwrap();

    // this can not fail
    let mut program = Program::from_str(&contents).unwrap();
    if let Err(e) = program.run() {
        eprintln!("{}", e.to_string());
        exit(-1);
    };
}
