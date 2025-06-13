#include "typer.h"

#include <assert.h>
#include <stdio.h>

#include "ast.h"
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

void resolve_types_in_expr(Expression *expr, Typer *typer);

Type resolve_types_in_block(Block *block, Typer *typer) {
    if (!block) { return TYPE_UNIT; }

    resolve_types_in_expr(block->head, typer);
    assert(block->head->inferred_type.resolved);

    if (block->tail) { return resolve_types_in_block(block->tail, typer); }

    return block->head->inferred_type.type;
}

void resolve_types_in_expr(Expression *expr, Typer *typer) {
    switch (expr->tag) {
    case EXPR_UNIT:
        expr->inferred_type.type = TYPE_UNIT;
        break;
    case EXPR_INTEGER:
        expr->inferred_type.type = TYPE_INT;
        break;
    case EXPR_VARIABLE:
        assert(typer->symbol_lookup[expr->variable.symbol].resolved);
        expr->inferred_type.type = typer->symbol_lookup[expr->variable.symbol].type;
        break;
    case EXPR_BINOP:
        resolve_types_in_expr(expr->binop.lhs, typer);
        assert(expr->binop.lhs->inferred_type.resolved);
        Type lhs_type = expr->binop.lhs->inferred_type.type;

        resolve_types_in_expr(expr->binop.rhs, typer);
        assert(expr->binop.rhs->inferred_type.resolved);
        Type rhs_type = expr->binop.rhs->inferred_type.type;

        switch (expr->binop.kind) {
        case BINOP_ASSIGN:
            assert(expr->binop.lhs->tag == EXPR_VARIABLE);
            /* TODO: mutability? */

            SymbolLookup lhs_type_lookup = typer->symbol_lookup[expr->binop.lhs->variable.symbol];
            assert(lhs_type_lookup.resolved);
            assert(lhs_type_lookup.type == rhs_type);

            expr->inferred_type.type = rhs_type;
            break;
        case BINOP_ADD:
            assert(lhs_type == TYPE_INT);
            assert(rhs_type == TYPE_INT);

            expr->inferred_type.type = TYPE_INT;
            break;
        }
    case EXPR_FUNCALL:
        /* TODO: funcall */
        break;
    case EXPR_VARDEF:
        resolve_types_in_expr(expr->vardef.expr, typer);
        assert(expr->vardef.expr->inferred_type.resolved);
        Type vardef_type = expr->vardef.expr->inferred_type.type;

        if (expr->vardef.type_annotation.annotated) {
            assert(expr->vardef.type_annotation.type == vardef_type);
        }

        typer->symbol_lookup[expr->vardef.symbol] = (SymbolLookup){
            .type = vardef_type,
            .resolved = true,
        };

        expr->inferred_type.type = TYPE_UNIT;
        break;
    case EXPR_BLOCK:
        Type block_type = resolve_types_in_block(expr->block, typer);
        expr->inferred_type.type = block_type;
        break;
    }
    expr->inferred_type.resolved = true;
}

void resolve_types_in_function(Function *fun, Typer *typer) {
    resolve_types_in_expr(fun->body, typer);
    assert(fun->body->inferred_type.resolved);
    Type inferred = fun->body->inferred_type.type;
    assert(typer->symbol_lookup[fun->symbol].resolved);
    assert(inferred == typer->symbol_lookup[fun->symbol].type);
}

void _resolve_types(Program *program, Typer *typer) {
    if (!program) { return; }
    resolve_types_in_function(&program->head, typer);
    _resolve_types(program->tail, typer);
}

void resolve_types_in_params(ParameterList *params, Typer *typer) {
    if (!params) { return; }

    assert(params->head.type_annotation.annotated);
    Type param_type = params->head.type_annotation.type;
    typer->symbol_lookup[params->head.symbol].type = param_type;
    typer->symbol_lookup[params->head.symbol].resolved = true;

    resolve_types_in_params(params->tail, typer);
}

void gather_function_prototype_information(Program *program, Typer *typer) {
    if (!program) { return; }

    assert(program->head.return_type_annotation.annotated);
    Type return_type = program->head.return_type_annotation.type;
    typer->symbol_lookup[program->head.symbol].type = return_type;
    typer->symbol_lookup[program->head.symbol].resolved = true;

    resolve_types_in_params(program->head.params, typer);

    gather_function_prototype_information(program->tail, typer);
}

void resolve_types(Program *program, SymbolId symbol_num) {
    Arena arena = {0};
    SymbolLookup *symbol_lookup = arena_alloc(&arena, symbol_num * sizeof(Type));
    Typer typer = {
        .arena = arena,
        .symbol_lookup = symbol_lookup,
        .cap = symbol_num,
    };

    gather_function_prototype_information(program, &typer);
    _resolve_types(program, &typer);

    arena_free(&typer.arena);
}
