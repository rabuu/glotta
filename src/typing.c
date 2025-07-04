#include "typing.h"

#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "ast.h"

typedef struct {
    bool resolved;
    AstType type;

    /* only for functions */
    AstParameters *params;
} Lookup;

typedef struct {
    Lookup *lookups;
    size_t lookup_count;
} Typer;

#if 0
#include "print.h"
static void debug_typer(Typer *typer) {
    for (size_t i = 0; i < typer->lookup_count; ++i) {
        if (typer->lookups[i].resolved) {
            printf("%zu: ", i);
            print_type(typer->lookups[i].type);
            printf(", %p\n", typer->lookups[i].params);
        }
    }
}
#endif /* if 0 */

static AstType resolve_types_in_expr(AstExpr *expr, Typer *typer);

static void resolve_types_in_args(AstArguments *args, AstParameters *params, Typer *typer) {
    assert(args->len == params->len);

    for (size_t i = 0; i < args->len; ++i) {
        AstType arg_type = resolve_types_in_expr(args->items[i], typer);
        AstType param_type = params->items[i].type_annotation.type;
        assert(arg_type == param_type);
    }
}

static AstType resolve_types_in_block(AstBlock *block, Typer *typer) {
    for (size_t i = 0; i < block->len; ++i) {
        resolve_types_in_expr(block->items[i], typer);
        assert(block->items[i]->inferred_type.resolved);
    }

    return block->items[block->len - 1]->inferred_type.type;
}

static AstType resolve_types_in_expr(AstExpr *expr, Typer *typer) {
    AstType inferred_type;
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
        AstType lhs_type = expr->binop.lhs->inferred_type.type;

        resolve_types_in_expr(expr->binop.rhs, typer);
        assert(expr->binop.rhs->inferred_type.resolved);
        AstType rhs_type = expr->binop.rhs->inferred_type.type;

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

        AstParameters *params = typer->lookups[expr->funcall.symbol].params;
        resolve_types_in_args(&expr->funcall.args, params, typer);
        inferred_type = typer->lookups[expr->funcall.symbol].type;
        break;
    case EXPR_VARDEF:
        resolve_types_in_expr(expr->vardef.expr, typer);
        assert(expr->vardef.expr->inferred_type.resolved);
        AstType vardef_type = expr->vardef.expr->inferred_type.type;

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
        AstType block_type = resolve_types_in_block(&expr->block, typer);
        inferred_type = block_type;
        break;
    }
    expr->inferred_type.type = inferred_type;
    expr->inferred_type.resolved = true;

    return inferred_type;
}

static void resolve_types_in_function(AstFunction *fun, Typer *typer) {
    AstType inferred = resolve_types_in_expr(fun->body, typer);
    assert(fun->body->inferred_type.resolved);
    assert(typer->lookups[fun->symbol].resolved);
    assert(inferred == typer->lookups[fun->symbol].type);
}

static void resolve_types_in_params(AstParameters *params, Typer *typer) {
    for (size_t i = 0; i < params->len; ++i) {
        AstParam param = params->items[i];

        assert(param.type_annotation.annotated);
        AstType param_type = param.type_annotation.type;
        typer->lookups[param.symbol].type = param_type;
        typer->lookups[param.symbol].resolved = true;
    }
}

static void gather_function_prototype_information(AstProgram *program, Typer *typer) {
    for (size_t i = 0; i < program->function_count; ++i) {
        AstFunction *fun = &program->functions[i];

        AstType return_type;
        if (fun->return_type_annotation.annotated) {
            return_type = fun->return_type_annotation.type;
        } else {
            return_type = TYPE_UNIT;
        }

        typer->lookups[fun->symbol].type = return_type;
        typer->lookups[fun->symbol].resolved = true;

        resolve_types_in_params(&fun->params, typer);
        typer->lookups[fun->symbol].params = &fun->params;
    }
}

void resolve_types(AstProgram *program, SymbolId symbol_num) {
    Lookup *symbol_lookup = calloc(symbol_num, sizeof(Lookup));
    Typer typer = {
        .lookups = symbol_lookup,
        .lookup_count = symbol_num,
    };

    gather_function_prototype_information(program, &typer);

    for (size_t i = 0; i < program->function_count; ++i) {
        resolve_types_in_function(&program->functions[i], &typer);
    }

    free(typer.lookups);
}
