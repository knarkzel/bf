use chumsky::prelude::*;
use gumdrop::Options;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Options)]
struct Args {
    // Input files to compile
    #[options(free, help = "brainfuck file to compile")]
    file: PathBuf,

    // Help option
    #[options(help = "print help message")]
    help: bool,
}

#[derive(Clone, Debug)]
enum Opcode {
    Left,
    Right,
    Add,
    Sub,
    Read,
    Write,
    Loop(Vec<Self>),
}

fn parser() -> impl Parser<char, Vec<Opcode>, Error = Simple<char>> {
    recursive(|bf| {
        choice((
            just('<').to(Opcode::Left),
            just('>').to(Opcode::Right),
            just('+').to(Opcode::Add),
            just('-').to(Opcode::Sub),
            just(',').to(Opcode::Read),
            just('.').to(Opcode::Write),
            bf.delimited_by(just('['), just(']')).map(Opcode::Loop),
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

fn main() {
    let args = Args::parse_args_default_or_exit();
    let body = read_to_string(args.file).expect("Error while reading file");
    let input = sanitize(body);
    let tokens = parser().parse(input).expect("Error while parsing");
    dbg!(&tokens);
}
