# bcc

Brainfuck to x86-64 assembly compiler and interpreter.

```bash
git clone https://github.com/knarkzel/bf
cd bf/
cargo install --path .

# Interpreter
bf run examples/beer.bf

# Compiler
function bcc() {
    basename=$(basename -- "$1")
    filename="${basename%.*}"
    bf build $1 $filename.asm
    clang -fno-pie -no-pie -nostdlib -fno-integrated-as -Wa,-msyntax=intel,-mnaked-reg -s $filename.asm -o $filename
    rm $filename.asm
    ls -alh $filename
}

bcc examples/beer.bf
```
