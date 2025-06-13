# Glotta 👅
My toy language.

Very much work in progress.

## Example program
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
- [Lexer](./src/lexer.c)
- [Parser](./src/parser.c)
- [Namer](./src/namer.c)
- [Typer](./src/typer.c)
