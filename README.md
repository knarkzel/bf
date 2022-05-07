# bcc

Brainfuck to x86-64 assembly compiler and interpreter.

# Running

```bash
git clone https://github.com/knarkzel/bcc
cd bcc/
cargo run --release -- run examples/beer.bf
```

# Assembling

```bash
cargo run --release -- build examples/beer.bf output.asm
clang -fno-pie -no-pie -nostdlib -fno-integrated-as -Wa,-msyntax=intel,-mnaked-reg -s output.asm -o output
./output
```
