#include "print.h"

#include <stdio.h>

#include "ast.h"
#include "util/slice.h"

void print_symbol(SymbolId id) {
    if (id != 0) { printf("#%zu", id); }
}

void print_slice(Slice slice) { printf("%.*s", (int)slice.len, slice.ptr); }
void print_slice_err(Slice slice) { fprintf(stderr, "%.*s", (int)slice.len, slice.ptr); }

void print_token(Token *token, SourceContext source) {
    char *tag = token_tag_to_str(token->tag);
    Slice lexeme = slice_from_location(source.buffer, token->loc);

    if (lexeme.len > 0) {
        printf("`");
        print_slice(lexeme);
        printf("` -> ");
    }
    printf("%s\n", tag);
}

void print_type(Type type) {
    switch (type) {
    case TYPE_UNIT:
        printf("Unit");
        break;
    case TYPE_INT:
        printf("Int");
        break;
    }
}

void print_inferred_type(InferredType inferred_type) {
    if (inferred_type.resolved) {
        printf("$");
        print_type(inferred_type.type);
    }
}

void print_binop(BinaryOp binop) {
    printf("(");
    switch (binop.kind) {
    case BINOP_ASSIGN:
        printf("ASSIGN");
        break;
    case BINOP_ADD:
        printf("ADD");
        break;
    }
    printf(" ");
    print_expression(binop.lhs);
    printf(" ");
    print_expression(binop.rhs);
    printf(")");
}

void print_vardef(VariableDefinition vardef) {
    printf("(");
    vardef.mutable ? printf("var") : printf("val");
    printf(" ");
    print_slice(vardef.name);
    print_symbol(vardef.symbol);
    printf(" :");

    if (vardef.type_annotation.annotated) {
        printf(" ");
        print_type(vardef.type_annotation.type);
        printf(" ");
    }

    printf("= ");
    print_expression(vardef.expr);
    printf(")");
}

void print_args(ArgumentList *args) {
    if (!args) { return; }

    print_expression(args->head);
    if (args->tail) {
        printf(", ");
        print_args(args->tail);
    }
}

void print_funcall(FunctionCall funcall) {
    print_slice(funcall.function);
    print_symbol(funcall.symbol);
    printf("(");
    print_args(funcall.args);
    printf(")");
}

void print_block_inner(Block *block) {
    if (!block) { return; }

    print_expression(block->head);
    if (block->tail) {
        printf("; ");
        print_block_inner(block->tail);
    }
}

void print_block(Block *block) {
    printf("{ ");
    print_block_inner(block);
    printf(" }");
}

void print_expression(Expression *expr) {
    switch (expr->tag) {
    case EXPR_UNIT:
        printf("unit");
        break;
    case EXPR_INTEGER:
        printf("%d", expr->integer);
        break;
    case EXPR_VARIABLE:
        print_slice(expr->variable.name);
        print_symbol(expr->variable.symbol);
        break;
    case EXPR_BINOP:
        print_binop(expr->binop);
        break;
    case EXPR_FUNCALL:
        print_funcall(expr->funcall);
        break;
    case EXPR_VARDEF:
        print_vardef(expr->vardef);
        break;
    case EXPR_BLOCK:
        print_block(expr->block);
        break;
    }
    print_inferred_type(expr->inferred_type);
}

void print_param(Parameter param) {
    if (param.mutable) { printf("var "); }

    print_slice(param.name);
    print_symbol(param.symbol);
    printf(": ");
    print_type(param.type_annotation.type);
}

void print_params(ParameterList *params) {
    if (!params) { return; }

    print_param(params->head);
    if (params->tail) {
        printf(", ");
        print_params(params->tail);
    }
}

void print_function(Function *fun) {
    printf("fun ");
    print_slice(fun->name);
    print_symbol(fun->symbol);
    printf("(");
    print_params(fun->params);
    printf("): ");
    print_type(fun->return_type_annotation.type);
    printf(" =\n    ");
    print_expression(fun->body);
    printf("\n");
}

void print_program(Program *program) {
    if (!program) { return; }

    print_function(&program->head);
    printf("\n");
    print_program(program->tail);
}
