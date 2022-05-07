# bcc

Brainfuck to x86-64 compiler and interpreter.

```bash
git clone https://github.com/knarkzel/bcc
cd bcc/
cargo run --release -- examples/beer.bf
```

# Assembling

```bash
cargo run --release -- examples/beer.bf
clang -fno-pie -no-pie -nostdlib -fno-integrated-as -Wa,-msyntax=intel,-mnaked-reg -s output.asm -o output
./output
```
