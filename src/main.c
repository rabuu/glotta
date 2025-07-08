#include <stdio.h>
#include <stdlib.h>

#include "arena.h"
#include "lexing.h"
#include "naming.h"
#include "parsing.h"
#include "print.h"
#include "project.h"
#include "source.h"
#include "typing.h"

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "Usage: glotta <SOURCE-FILE.glotta | PROJECT-DIR>\n");
        exit(1);
    }

    char *input_path = argv[1];
    char path[4096];
    if (!realpath(input_path, path)) {
        return -1;
    }
    Project project = read_project(path);

    for (size_t i = 0; i < project.module_count; ++i) {
        print_module(&project.modules[i]);
    }

    return 0;

    SourceContext source = read_source_from_file(path);
    printf("------- SOURCE ---------\n");
    printf("%s\n", source.buffer);

    Lexer lexer = lexer_init(source);

#if 0
    printf("\n------- LEXING ---------\n");
    Token tok;
    do {
        tok = lexer_next(&lexer);
        print_token(&tok, source);
    } while (tok.tag != TOK_EOF);
    printf("------------------------\n");
    lexer.index = 0;
#endif

    printf("\n--------- AST ----------\n");
    Arena ast_arena = {0};
    AST_Program ast = parse_program(&lexer, &ast_arena);
    print_program(&ast);

    printf("\n-------- NAMING --------\n");
    SymbolId symbol_num = resolve_names(&ast);
    print_program(&ast);

    printf("\n-------- TYPING ----------\n");
    resolve_types(&ast, symbol_num);
    print_program(&ast);

    project_free(&project);

    return 0;
}
