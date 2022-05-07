# bcc

Brainfuck to x86-64 assembly compiler and interpreter.

```bash
cargo install --git https://github.com/knarkzel/bcc

# Interprete
bf run examples/beer.bf

# Compile
bf build examples/beer.bf output.asm
clang -fno-pie -no-pie -nostdlib -fno-integrated-as -Wa,-msyntax=intel,-mnaked-reg -s output.asm -o output
./output
```
