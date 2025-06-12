#include "typer.h"

#include <assert.h>
#include <stdio.h>

#include "ast.h"
#include "print.h"
#include "util/arena.h"

typedef struct {
    bool resolved;
    Type type;
} SymbolLookup;

typedef struct {
    Arena arena;

    SymbolLookup *symbol_lookup;
    size_t cap;
} Typer;

void print_symbol_lookup(Typer *typer) {
    for (size_t i = 0; i < typer->cap; ++i) {
        if (typer->symbol_lookup[i].resolved) {
            printf("%zu: ", i);
            print_type(typer->symbol_lookup[i].type);
            printf("\n");
        }
    }
}

void resolve_types_in_function(Function *fun, Typer *typer) { return; }

void _resolve_types(Program *program, Typer *typer) {
    if (!program) { return; }
    resolve_types_in_function(&program->head, typer);
    _resolve_types(program->tail, typer);
}

void resolve_function_return_types(Program *program, Typer *typer) {
    if (!program) { return; }

    assert(program->head.return_type_annotation.annotated);
    Type return_type = program->head.return_type_annotation.type;
    typer->symbol_lookup[program->head.symbol].type = return_type;
    typer->symbol_lookup[program->head.symbol].resolved = true;

    resolve_function_return_types(program->tail, typer);
}

void resolve_types(Program *program, SymbolId symbol_num) {
    Arena arena = {0};
    SymbolLookup *symbol_lookup = arena_alloc(&arena, symbol_num * sizeof(Type));
    Typer typer = {
        .arena = arena,
        .symbol_lookup = symbol_lookup,
        .cap = symbol_num,
    };

    print_symbol_lookup(&typer);

    resolve_function_return_types(program, &typer);
    _resolve_types(program, &typer);

    print_symbol_lookup(&typer);

    arena_free(&typer.arena);
}
