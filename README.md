# Glotta 👅
My toy language.

Very much work in progress.

## Example program
This is the minimal example I work on to compile at the moment:
```
fun main(): Int = {
    val a := add(1, 2);
    var b : Int = 12;
    b = a + b;
    b + 1
}

// This is a comment!
fun add(x: Int, y: Int): Int = x + y
```

## Progess
- [Lexing](./src/lexing.c)
- [Parsing](./src/parsing.c)
- [Naming](./src/naming.c)
- [Typing](./src/typing.c)
