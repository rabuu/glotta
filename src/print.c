#include "print.h"

#include <stdio.h>

#include "ast.h"
#include "strslice.h"

static void print_symbol(SymbolId id) {
    if (id != 0) { printf("#%zu", id); }
}

void print_strslice(StrSlice slice) { printf("%.*s", (int)slice.len, slice.ptr); }
void print_strslice_err(StrSlice slice) { fprintf(stderr, "%.*s", (int)slice.len, slice.ptr); }

void print_token(Token *token, SourceContext source) {
    char *tag = token_tag_to_str(token->tag);
    StrSlice lexeme = strslice_from_loc(source.buffer, token->loc);

    if (lexeme.len > 0) {
        printf("`");
        print_strslice(lexeme);
        printf("` -> ");
    }
    printf("%s\n", tag);
}

void print_type(AST_Type type) {
    switch (type) {
    case TYPE_UNIT:
        printf("Unit");
        break;
    case TYPE_INT:
        printf("Int");
        break;
    }
}

static void print_inferred_type(AST_InferredType inferred_type) {
    if (inferred_type.resolved) {
        printf("$");
        print_type(inferred_type.type);
    }
}

static void print_binop(AST_BinaryOp binop) {
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

static void print_vardef(AST_VarDef vardef) {
    printf("(");
    vardef.mutable ? printf("var") : printf("val");
    printf(" ");
    print_strslice(vardef.name);
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

static void print_funcall(AST_FunCall funcall) {
    printf(".");
    print_strslice(funcall.function);
    print_symbol(funcall.symbol);
    printf("(");
    for (size_t i = 0; i < funcall.prefix_args.len; ++i) {
        if (i != 0) { printf(", "); }
        print_expression(funcall.prefix_args.items[i]);
    }
    if (funcall.prefix_args.len > 0 && funcall.args.len > 0) {
        printf(", ");
    }
    for (size_t i = 0; i < funcall.args.len; ++i) {
        if (i != 0) { printf(", "); }
        print_expression(funcall.args.items[i]);
    }
    printf(")");
}

static void print_block(AST_Block block) {
    printf("{ ");
    for (size_t i = 0; i < block.len; ++i) {
        if (i != 0) { printf("; "); }
        print_expression(block.items[i]);
    }
    printf(" }");
}

void print_expression(AST_Expr *expr) {
    switch (expr->tag) {
    case EXPR_UNIT:
        printf("unit");
        break;
    case EXPR_INTEGER:
        printf("%d", expr->integer);
        break;
    case EXPR_VARIABLE:
        print_strslice(expr->variable.name);
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

static void print_param(AST_Param param) {
    if (param.mutable) { printf("var "); }

    print_strslice(param.name);
    print_symbol(param.symbol);
    printf(": ");
    print_type(param.type_annotation.type);
}

void print_function(AST_Function *fun) {
    printf("fun ");
    print_strslice(fun->name);
    print_symbol(fun->symbol);
    printf("(");
    for (size_t i = 0; i < fun->params.len; ++i) {
        if (i != 0) { printf(", "); }
        print_param(fun->params.items[i]);
    }
    printf("): ");
    print_type(fun->return_type_annotation.type);
    printf(" =\n    ");
    print_expression(fun->body);
    printf("\n");
}

void print_program(AST_Program *program) {
    for (size_t i = 0; i < program->function_count; ++i) {
        if (i != 0) { printf("\n"); }
        print_function(&program->functions[i]);
    }
}
