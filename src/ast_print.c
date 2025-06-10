#include "ast_print.h"

#include <stdio.h>

#include "ast.h"
#include "util/slice.h"

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

void print_binop(BinaryOp binop) {
    printf("(");
    switch (binop.kind) {
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

void print_assignment(Assignment ass) {
    ass.mutable ? printf("var") : printf("val");
    printf(" ");
    slice_print(ass.name);
    printf(" :");

    if (ass.type_annotation.annotated) {
        printf(" ");
        print_type(ass.type_annotation.type);
        printf(" ");
    }

    printf("= ");
    print_expression(ass.expr);
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
    slice_print(funcall.function);
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
        slice_print(expr->variable);
        break;
    case EXPR_BINOP:
        print_binop(expr->binop);
        break;
    case EXPR_FUNCALL:
        print_funcall(expr->funcall);
        break;
    case EXPR_ASSIGNMENT:
        print_assignment(expr->assignment);
        break;
    case EXPR_BLOCK:
        print_block(expr->block);
        break;
    }
}

void print_param(Parameter param) {
    if (param.mutable) { printf("var "); }

    slice_print(param.name);
    printf(": ");
    print_type(param.type);
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
    slice_print(fun->name);
    printf("(");
    print_params(fun->params);
    printf("): ");
    print_type(fun->return_type);
    printf(" =\n    ");
    print_expression(fun->body);
    printf("\n");
}
