#include <stdio.h>
#include <stdlib.h>

#include "lexer.h"
#include "namer.h"
#include "parser.h"
#include "print.h"
#include "source.h"
#include "typer.h"
#include "util/arena.h"

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "Usage: glotta SOURCE-FILE.glotta\n");
        exit(1);
    }

    char *path = argv[1];
    printf("PATH: %s\n------------------------\n", path);

    SourceContext source = read_source_from_file(path);
    printf("%s\n------------------------\n", source.buffer);

    Lexer lexer = lexer_init(source);

    Token tok;
    do {
        tok = lexer_next(&lexer);
        print_token(&tok, source);
    } while (tok.tag != TOK_EOF);
    printf("------------------------\n");

    lexer.index = 0;

    Arena ast_arena = {0};
    Program ast = parse_program(&lexer, &ast_arena);
    print_program(&ast);
    printf("------------------------\n");

    SymbolId symbol_num = resolve_names(&ast);
    print_program(&ast);
    printf("--------------------------\n");

    resolve_types(&ast, symbol_num);
    print_program(&ast);

    return 0;
}
