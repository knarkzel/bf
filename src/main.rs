#![feature(slice_group_by)]

use anyhow::Error;
use chumsky::prelude::*;
use fehler::throws;
use gumdrop::Options;
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::{stdin, Read};
use std::path::PathBuf;

// Arguments
#[derive(Options)]
struct Args {
    // Help option
    #[options(help = "print help message")]
    help: bool,

    // Command
    #[options(command)]
    command: Option<Command>,
}

#[derive(Options)]
enum Command {
    Build(BuildOpts),
    Run(RunOpts),
}

#[derive(Options)]
struct BuildOpts {
    #[options(free, help = "brainfuck file to compile")]
    input: PathBuf,

    #[options(free, help = "output name")]
    output: PathBuf,
}

#[derive(Options)]
struct RunOpts {
    #[options(free, help = "brainfuck file to run")]
    input: PathBuf,
}

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

fn parser() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    recursive(|bf| {
        choice((
            just('<').to(Token::Left),
            just('>').to(Token::Right),
            just('+').to(Token::Add),
            just('-').to(Token::Sub),
            just(',').to(Token::Read),
            just('.').to(Token::Write),
            bf.delimited_by(just('['), just(']')).map(Token::Loop),
        ))
        .repeated()
    })
}

fn sanitize(input: String) -> String {
    input
        .chars()
        .filter(|it| "<>+-,.[]".chars().any(|token| *it == token))
        .collect()
}

// Interpreter
struct Interpreter {
    memory: [u8; 65536],
    data_pointer: usize,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            memory: [0; 65536],
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
    fn new(file_name: &PathBuf) -> Self {
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

#[throws]
fn main() {
    let args = Args::parse_args_default_or_exit();
    match args.command.expect("Expected build or run command") {
        Command::Build(opts) => {
            let body = read_to_string(opts.input)?;
            let input = sanitize(body);
            let tokens = parser().parse(input).expect("Error while parsing");
            let mut assembler = Assembler::new(&opts.output)?;
            assembler.assembly(&tokens)?;
        }
        Command::Run(opts) => {
            let body = read_to_string(opts.input)?;
            let input = sanitize(body);
            let tokens = parser().parse(input).expect("Error while parsing");
            let mut interpreter = Interpreter::new();
            interpreter.interpret(&tokens);
        },
    }
}
