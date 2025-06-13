#include "typer.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

#include "ast.h"
#include "print.h"

typedef struct {
    bool resolved;
    Type type;

    /* only for functions */
    ParameterList *params;
} Lookup;

typedef struct {
    Lookup *lookups;
    size_t cap;
} Typer;

void debug_typer(Typer *typer) {
    for (size_t i = 0; i < typer->cap; ++i) {
        if (typer->lookups[i].resolved) {
            printf("%zu: ", i);
            print_type(typer->lookups[i].type);
            printf(", %p\n", typer->lookups[i].params);
        }
    }
}

Type resolve_types_in_expr(Expression *expr, Typer *typer);

void resolve_types_in_args(ArgumentList *args, ParameterList *params, Typer *typer) {
    if (!args) {
        assert(!params);
        return;
    } else {
        assert(params);
    }

    Type param_type = params->head.type_annotation.type;
    Type arg_type = resolve_types_in_expr(args->head, typer);
    assert(param_type == arg_type);

    resolve_types_in_args(args->tail, params->tail, typer);
}

Type resolve_types_in_block(Block *block, Typer *typer) {
    if (!block) { return TYPE_UNIT; }

    Type expr_type = resolve_types_in_expr(block->head, typer);
    assert(block->head->inferred_type.resolved);

    if (block->tail) { return resolve_types_in_block(block->tail, typer); }

    return expr_type;
}

Type resolve_types_in_expr(Expression *expr, Typer *typer) {
    Type inferred_type;
    switch (expr->tag) {
    case EXPR_UNIT:
        inferred_type = TYPE_UNIT;
        break;
    case EXPR_INTEGER:
        inferred_type = TYPE_INT;
        break;
    case EXPR_VARIABLE:
        assert(typer->lookups[expr->variable.symbol].resolved);
        inferred_type = typer->lookups[expr->variable.symbol].type;
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

            Lookup lhs_type_lookup = typer->lookups[expr->binop.lhs->variable.symbol];
            assert(lhs_type_lookup.resolved);
            assert(lhs_type_lookup.type == rhs_type);

            inferred_type = rhs_type;
            break;
        case BINOP_ADD:
            assert(lhs_type == TYPE_INT);
            assert(rhs_type == TYPE_INT);
            inferred_type = TYPE_INT;
            break;
        }
        break;
    case EXPR_FUNCALL:
        assert(typer->lookups[expr->funcall.symbol].resolved);

        ParameterList *params = typer->lookups[expr->funcall.symbol].params;
        resolve_types_in_args(expr->funcall.args, params, typer);
        inferred_type = typer->lookups[expr->funcall.symbol].type;
        break;
    case EXPR_VARDEF:
        resolve_types_in_expr(expr->vardef.expr, typer);
        assert(expr->vardef.expr->inferred_type.resolved);
        Type vardef_type = expr->vardef.expr->inferred_type.type;

        if (expr->vardef.type_annotation.annotated) {
            assert(expr->vardef.type_annotation.type == vardef_type);
        }

        typer->lookups[expr->vardef.symbol] = (Lookup){
            .type = vardef_type,
            .resolved = true,
        };

        inferred_type = TYPE_UNIT;
        break;
    case EXPR_BLOCK:
        Type block_type = resolve_types_in_block(expr->block, typer);
        inferred_type = block_type;
        break;
    }
    expr->inferred_type.type = inferred_type;
    expr->inferred_type.resolved = true;

    return inferred_type;
}

void resolve_types_in_function(Function *fun, Typer *typer) {
    Type inferred = resolve_types_in_expr(fun->body, typer);
    assert(fun->body->inferred_type.resolved);
    assert(typer->lookups[fun->symbol].resolved);
    assert(inferred == typer->lookups[fun->symbol].type);
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
    typer->lookups[params->head.symbol].type = param_type;
    typer->lookups[params->head.symbol].resolved = true;

    resolve_types_in_params(params->tail, typer);
}

void gather_function_prototype_information(Program *program, Typer *typer) {
    if (!program) { return; }

    assert(program->head.return_type_annotation.annotated);
    Type return_type = program->head.return_type_annotation.type;
    typer->lookups[program->head.symbol].type = return_type;
    typer->lookups[program->head.symbol].resolved = true;

    resolve_types_in_params(program->head.params, typer);
    typer->lookups[program->head.symbol].params = program->head.params;

    gather_function_prototype_information(program->tail, typer);
}

void resolve_types(Program *program, SymbolId symbol_num) {
    Lookup *symbol_lookup = calloc(symbol_num, sizeof(Lookup));
    Typer typer = {
        .lookups = symbol_lookup,
        .cap = symbol_num,
    };

    gather_function_prototype_information(program, &typer);
    _resolve_types(program, &typer);

    free(typer.lookups);
}
