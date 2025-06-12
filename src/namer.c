#include "namer.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ast.h"
#include "print.h"
#include "util/arena.h"

/* forward declarations */
void resolve_names_in_expr(Expression *expr, Scope *scope, SymbolId *new_symbol, Arena *a);

Scope *scope_fork(Scope *parent, Arena *a) {
    Scope *scope = (Scope *)arena_alloc(a, sizeof(Scope));

    scope->parent = parent;
    scope->count = 0;
    scope->cap = DEFAULT_SYMBOL_TABLE_CAP;
    scope->symbols = (Symbol *)arena_alloc(a, scope->cap * sizeof(Symbol));

    return scope;
}

void scope_add_symbol(Scope *scope, Symbol symbol, Arena *a) {
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

SymbolId scope_lookup_single(Scope *scope, Slice name, SymbolKind kind) {
    for (size_t i = 0; i < scope->count; ++i) {
        size_t idx = scope->count - i - 1;
        Symbol lookup = scope->symbols[idx];
        if (lookup.kind == kind && slice_eq(lookup.name, name)) { return lookup.id; }
    }

    return 0;
}

SymbolId scope_lookup(Scope *scope, Slice name, SymbolKind kind) {
    if (!scope) { return 0; }

    SymbolId this_scope = scope_lookup_single(scope, name, kind);
    if (this_scope != 0) { return this_scope; }

    return scope_lookup(scope->parent, name, kind);
}

void scope_add_params(Scope *scope, ParameterList *params, SymbolId *new_symbol, Arena *a) {
    if (!params) { return; }

    if (scope_lookup_single(scope, params->head.name, SYMBOL_VARIABLE)) {
        fprintf(stderr, "ERROR: Parameter `");
        print_slice_err(params->head.name);
        fprintf(stderr, "` already defined.\n");
        exit(1);
    }

    Symbol symbol = {
        .kind = SYMBOL_VARIABLE,
        .name = params->head.name,
        .id = *new_symbol,
    };
    params->head.symbol = *new_symbol;
    *new_symbol += 1;

    scope_add_symbol(scope, symbol, a);
    scope_add_params(scope, params->tail, new_symbol, a);
}

void resolve_names_in_block(Block *block, Scope *scope, SymbolId *new_symbol, Arena *a) {
    if (!block) { return; }
    resolve_names_in_expr(block->head, scope, new_symbol, a);
    resolve_names_in_block(block->tail, scope, new_symbol, a);
}

void resolve_names_in_expr(Expression *expr, Scope *scope, SymbolId *new_symbol, Arena *a) {
    switch (expr->tag) {
    case EXPR_BLOCK:
        Scope *block_scope = scope_fork(scope, a);
        resolve_names_in_block(expr->block, block_scope, new_symbol, a);
        break;
    case EXPR_VARIABLE:
        SymbolId symbol_var = scope_lookup(scope, expr->variable.name, SYMBOL_VARIABLE);
        if (!symbol_var) {
            fprintf(stderr, "ERROR: Variable name `");
            print_slice_err(expr->variable.name);
            fprintf(stderr, "` is not bound.\n");
            exit(1);
        }
        expr->variable.symbol = symbol_var;
        break;
    case EXPR_FUNCALL:
        SymbolId symbol_funcall = scope_lookup(scope, expr->funcall.function, SYMBOL_FUNCTION);
        if (!symbol_funcall) {
            fprintf(stderr, "ERROR: Function name `");
            print_slice_err(expr->funcall.function);
            fprintf(stderr, "` is not bound.\n");
            exit(1);
        }
        expr->funcall.symbol = symbol_funcall;
        break;
    case EXPR_VARDEF:
        Symbol symbol_vardef = {
            .kind = SYMBOL_VARIABLE,
            .name = expr->vardef.name,
            .id = *new_symbol,
        };
        expr->vardef.symbol = *new_symbol;
        *new_symbol += 1;
        scope_add_symbol(scope, symbol_vardef, a);
        break;
    case EXPR_BINOP:
        resolve_names_in_expr(expr->binop.lhs, scope, new_symbol, a);
        resolve_names_in_expr(expr->binop.rhs, scope, new_symbol, a);
        break;
    case EXPR_UNIT:
    case EXPR_INTEGER:
        break;
    }
}

void resolve_names_in_function(Function *fun, Scope *scope, SymbolId *new_symbol, Arena *a) {
    if (scope_lookup(scope, fun->name, SYMBOL_FUNCTION)) {
        fprintf(stderr, "ERROR: Function `");
        print_slice_err(fun->name);
        fprintf(stderr, "` already defined.\n");
        exit(1);
    }

    Symbol function_symbol = {
        .kind = SYMBOL_FUNCTION,
        .name = fun->name,
        .id = *new_symbol,
    };
    fun->symbol = *new_symbol;
    *new_symbol += 1;
    scope_add_symbol(scope, function_symbol, a);

    Scope *param_scope = scope_fork(scope, a);
    scope_add_params(param_scope, fun->params, new_symbol, a);

    Scope *function_scope = scope_fork(param_scope, a);
    resolve_names_in_expr(fun->body, function_scope, new_symbol, a);
}

void _resolve_names(Program *program, Scope *scope, SymbolId *new_symbol, Arena *a) {
    if (!program) { return; }
    resolve_names_in_function(&program->head, scope, new_symbol, a);
    _resolve_names(program->tail, scope, new_symbol, a);
}

void resolve_names(Program *program) {
    Arena namer_arena = {0};
    Scope *global_scope = scope_fork(nullptr, &namer_arena);
    SymbolId new_symbol = 1;
    _resolve_names(program, global_scope, &new_symbol, &namer_arena);
    arena_free(&namer_arena);
}
