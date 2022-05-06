use chumsky::prelude::*;
use gumdrop::Options;
use std::fs::read_to_string;
use std::io::{stdin, Read};
use std::path::PathBuf;

// Arguments
#[derive(Options)]
struct Args {
    // Input files to compile
    #[options(free, help = "brainfuck file to compile")]
    file: PathBuf,

    // Help option
    #[options(help = "print help message")]
    help: bool,
}

// Parser
#[derive(Clone, Debug)]
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
#[derive(Debug)]
struct State {
    memory: [u8; 65536],
    data_pointer: usize,
}

impl State {
    fn new() -> Self {
        Self {
            memory: [0; 65536],
            data_pointer: 0,
        }
    }

    fn interpret(&mut self, tokens: &mut [Token]) {
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
                Token::Loop(tokens) => while self.memory[self.data_pointer] > 0 { self.interpret(tokens) },
            }
        }
    }
}

fn main() {
    let args = Args::parse_args_default_or_exit();
    let body = read_to_string(args.file).expect("Error while reading file");
    let input = sanitize(body);
    let mut tokens = parser().parse(input).expect("Error while parsing");
    let mut state = State::new();
    state.interpret(&mut tokens);
}
