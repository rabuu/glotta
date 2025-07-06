#include "naming.h"

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ast.h"
#include "print.h"
#include "util/arena.h"
#include "util/slice.h"

#define DEFAULT_SYMBOL_TABLE_CAP 4

typedef enum {
    SYMBOL_FUNCTION,
    SYMBOL_VARIABLE,
} SymbolKind;

typedef struct Scope Scope;
typedef struct {
    SymbolKind kind;
    SymbolId id;
    StrSlice name;
} Symbol;

struct Scope {
    struct Scope *parent;

    Symbol *symbols;

    size_t count;
    size_t cap;
};

typedef struct {
    Arena arena;
    SymbolId new_symbol;
} Namer;

static Scope *scope_fork(Scope *parent, Arena *a) {
    Scope *scope = (Scope *)arena_alloc(a, sizeof(Scope));

    scope->parent = parent;
    scope->count = 0;
    scope->cap = DEFAULT_SYMBOL_TABLE_CAP;
    scope->symbols = (Symbol *)arena_alloc(a, scope->cap * sizeof(Symbol));

    return scope;
}

static void scope_add_symbol(Scope *scope, Symbol symbol, Arena *a) {
    if (scope->count == scope->cap) {
        size_t new_cap = scope->cap * 2;
        if (new_cap == 0) { new_cap = DEFAULT_SYMBOL_TABLE_CAP; }

        /* maybe implement a realloc for arena? */
        Symbol *new_symbols = (Symbol *)arena_alloc(a, new_cap * sizeof(Symbol));
        memcpy(new_symbols, scope->symbols, scope->count * sizeof(Symbol));
        scope->symbols = new_symbols;
        scope->cap = new_cap;
    }

    scope->symbols[scope->count] = symbol;
    scope->count++;
}

static SymbolId scope_lookup_single(Scope *scope, StrSlice name, SymbolKind kind) {
    for (size_t i = 0; i < scope->count; ++i) {
        size_t idx = scope->count - i - 1;
        Symbol lookup = scope->symbols[idx];
        if (lookup.kind == kind && strslice_eq(lookup.name, name)) { return lookup.id; }
    }

    return 0;
}

static SymbolId scope_lookup(Scope *scope, StrSlice name, SymbolKind kind) {
    if (!scope) { return 0; }

    SymbolId this_scope = scope_lookup_single(scope, name, kind);
    if (this_scope != 0) { return this_scope; }

    return scope_lookup(scope->parent, name, kind);
}

static void resolve_names_in_expr(AST_Expr *expr, Scope *scope, Namer *namer);

static void resolve_names_in_params(AST_Parameters *params, Scope *scope, Namer *namer) {
    for (size_t i = 0; i < params->len; ++i) {
        AST_Param *param = &params->items[i];

        if (scope_lookup_single(scope, param->name, SYMBOL_VARIABLE)) {
            fprintf(stderr, "ERROR: Parameter `");
            print_strslice_err(param->name);
            fprintf(stderr, "` already defined.\n");
            exit(1);
        }

        Symbol symbol = {
            .kind = SYMBOL_VARIABLE,
            .name = param->name,
            .id = namer->new_symbol,
        };
        param->symbol = namer->new_symbol;
        namer->new_symbol += 1;

        scope_add_symbol(scope, symbol, &namer->arena);
    }
}

static void resolve_names_in_expr(AST_Expr *expr, Scope *scope, Namer *namer) {
    switch (expr->tag) {
    case EXPR_BLOCK:
        Scope *block_scope = scope_fork(scope, &namer->arena);
        for (size_t i = 0; i < expr->block.len; ++i) {
            resolve_names_in_expr(expr->block.items[i], block_scope, namer);
        }
        break;
    case EXPR_VARIABLE:
        SymbolId symbol_var = scope_lookup(scope, expr->variable.name, SYMBOL_VARIABLE);
        if (!symbol_var) {
            fprintf(stderr, "ERROR: Variable name `");
            print_strslice_err(expr->variable.name);
            fprintf(stderr, "` is not bound.\n");
            exit(1);
        }
        expr->variable.symbol = symbol_var;
        break;
    case EXPR_FUNCALL:
        SymbolId symbol_funcall = scope_lookup(scope, expr->funcall.function, SYMBOL_FUNCTION);
        if (!symbol_funcall) {
            fprintf(stderr, "ERROR: Function name `");
            print_strslice_err(expr->funcall.function);
            fprintf(stderr, "` is not bound.\n");
            exit(1);
        }
        expr->funcall.symbol = symbol_funcall;

        for (size_t i = 0; i < expr->funcall.args.len; ++i) {
            resolve_names_in_expr(expr->funcall.args.items[i], scope, namer);
        }
        break;
    case EXPR_VARDEF:
        resolve_names_in_expr(expr->vardef.expr, scope, namer);

        Symbol symbol_vardef = {
            .kind = SYMBOL_VARIABLE,
            .name = expr->vardef.name,
            .id = namer->new_symbol,
        };
        expr->vardef.symbol = namer->new_symbol;
        namer->new_symbol += 1;
        scope_add_symbol(scope, symbol_vardef, &namer->arena);
        break;
    case EXPR_BINOP:
        resolve_names_in_expr(expr->binop.lhs, scope, namer);
        resolve_names_in_expr(expr->binop.rhs, scope, namer);
        break;
    case EXPR_UNIT:
    case EXPR_INTEGER:
        break;
    }
}

static void resolve_names_in_function(AST_Function *fun, Scope *scope, Namer *namer) {
    Scope *param_scope = scope_fork(scope, &namer->arena);
    resolve_names_in_params(&fun->params, param_scope, namer);

    Scope *function_scope = scope_fork(param_scope, &namer->arena);
    resolve_names_in_expr(fun->body, function_scope, namer);
}

static void resolve_top_level_names(AST_Program *program, Scope *scope, Namer *namer) {
    for (size_t i = 0; i < program->function_count; ++i) {
        AST_Function *fun = &program->functions[i];

        if (scope_lookup(scope, fun->name, SYMBOL_FUNCTION)) {
            fprintf(stderr, "ERROR: Function `");
            print_strslice_err(fun->name);
            fprintf(stderr, "` already defined.\n");
            exit(1);
        }

        Symbol function_symbol = {
            .kind = SYMBOL_FUNCTION,
            .name = fun->name,
            .id = namer->new_symbol,
        };

        fun->symbol = namer->new_symbol;
        namer->new_symbol += 1;
        scope_add_symbol(scope, function_symbol, &namer->arena);
    }
}

SymbolId resolve_names(AST_Program *program) {
    Namer namer = {
        .arena = {0},
        .new_symbol = 1,
    };

    Scope *global_scope = scope_fork(nullptr, &namer.arena);

    resolve_top_level_names(program, global_scope, &namer);

    for (size_t i = 0; i < program->function_count; ++i) {
        resolve_names_in_function(&program->functions[i], global_scope, &namer);
    }

    arena_free(&namer.arena);

    return namer.new_symbol;
}
