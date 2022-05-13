#![feature(slice_group_by)]

use anyhow::Error;
use fehler::throws;
use nom::{
    branch::alt, bytes::complete::tag, combinator::map, multi::many0, sequence::delimited, IResult,
};
use std::{
    fs::{read_to_string, File},
    io::{stdin, BufWriter, Read, Write},
};

// Parser
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Left,
    Right,
    Add,
    Sub,
    Read,
    Write,
    Loop(Vec<Self>),
}

fn parse(input: &str) -> IResult<&str, Vec<Token>> {
    many0(alt((
        map(tag("<"), |_| Token::Left),
        map(tag(">"), |_| Token::Right),
        map(tag("+"), |_| Token::Add),
        map(tag("-"), |_| Token::Sub),
        map(tag(","), |_| Token::Read),
        map(tag("."), |_| Token::Write),
        map(delimited(tag("["), parse, tag("]")), Token::Loop),
    )))(input)
}

// Interpreter
struct Interpreter {
    memory: [u8; 30000],
    data_pointer: usize,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            memory: [0; 30000],
            data_pointer: 0,
        }
    }

    fn interpret(&mut self, tokens: &[Token]) {
        for token in tokens {
            match token {
                Token::Left => self.data_pointer -= 1,
                Token::Right => self.data_pointer += 1,
                Token::Add => self.memory[self.data_pointer] += 1,
                Token::Sub => self.memory[self.data_pointer] -= 1,
                Token::Read => {
                    if let Some(Ok(byte)) = stdin().bytes().next() {
                        self.memory[self.data_pointer] = byte;
                    }
                }
                Token::Write => print!("{}", self.memory[self.data_pointer] as char),
                Token::Loop(tokens) => {
                    while self.memory[self.data_pointer] > 0 {
                        self.interpret(tokens)
                    }
                }
            }
        }
    }
}

// Output assembly
const HEADER: &str = ".data
.bss
.lcomm ARRAY, 30000
.text
.global _start
_start:
mov r12, offset ARRAY";

const FOOTER: &str = "mov rax, 60
mov rdi, 9
syscall";

const READ: &str = "mov rax, 0
mov rdi, 0
mov rsi, r12
mov rdx, 1
syscall";

const WRITE: &str = "mov rax, 1
mov rdi, 1
mov rsi, r12
mov rdx, 1
syscall";

struct Assembler {
    index: usize,
    output: BufWriter<File>,
}

impl Assembler {
    #[throws]
    fn new(file_name: &str) -> Self {
        // Open .asm file
        let file = File::create(file_name)?;
        let output = BufWriter::new(file);
        Self { index: 0, output }
    }

    #[throws]
    fn dump(&mut self, tokens: &[Token]) {
        // Coalesce similar tokens into (count, token)
        for group in tokens.group_by(|a, b| a == b) {
            let count = group.len();
            if let Some(token) = group.first() {
                match token {
                    Token::Left => writeln!(&mut self.output, "sub r12, {count}")?,
                    Token::Right => writeln!(&mut self.output, "add r12, {count}")?,
                    Token::Add => writeln!(&mut self.output, "addb [r12], {count}")?,
                    Token::Sub => writeln!(&mut self.output, "subb [r12], {count}")?,
                    Token::Read => writeln!(&mut self.output, "{READ}")?,
                    Token::Write => writeln!(&mut self.output, "{WRITE}")?,
                    Token::Loop(tokens) => {
                        let index = self.index;
                        self.index += 1;
                        writeln!(
                            &mut self.output,
                            "cmpb [r12], 0\nje LOOP_END_{index}\nLOOP_START_{index}:"
                        )?;
                        self.dump(&tokens)?;
                        writeln!(
                            &mut self.output,
                            "cmpb [r12], 0\njne LOOP_START_{index}\nLOOP_END_{index}:"
                        )?;
                    }
                }
            }
        }
    }

    #[throws]
    fn assembly(&mut self, tokens: &[Token]) {
        writeln!(&mut self.output, "{HEADER}")?;
        self.dump(tokens)?;
        writeln!(&mut self.output, "{FOOTER}")?;
    }
}

fn tokens(input: &str) -> Vec<Token> {
    let body = read_to_string(input)
        .expect("Invalid file name")
        .chars()
        .filter(|it| "<>+-,.[]".chars().any(|token| *it == token))
        .collect::<String>();
    parse(&body).expect("Error while parsing").1
}

#[throws]
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let slice = args.iter().map(String::as_str).collect::<Vec<_>>();
    match slice[..] {
        ["run", input] => {
            let tokens = tokens(input);
            Interpreter::new().interpret(&tokens);
        }
        ["run", ..] => println!("bf: run <input>"),
        ["build", input, output] => {
            let tokens = tokens(input);
            Assembler::new(output)?.assembly(&tokens)?;
        }
        ["build", ..] => println!("bf: build <input> <output>"),
        _ => println!("bf: run, build"),
    }
}
