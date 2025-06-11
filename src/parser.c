/* PARSER
 * ------
 * Pratt parsing: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
 */

#include "parser.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ast.h"
#include "lexer.h"
#include "source.h"
#include "util/arena.h"
#include "util/slice.h"

void error_prefix(size_t index, SourceContext source) {
    FileLocation floc = file_location(index, source);
    fprintf(stderr, "ERROR[%s:%zu:%zu]: ", source.filename, floc.row, floc.column);
}

void expect(Token *token, TokenTag tag, SourceContext source) {
    if (token->tag != tag) {
        char *expected = token_tag_to_str(tag);
        char *but_got = token_tag_to_str(token->tag);

        error_prefix(token->loc.start, source);
        fprintf(stderr, "Expected `%s` but got `%s`.\n", expected, but_got);
        exit(1);
    }
}

Expression *expr_init(Arena *a) { return arena_alloc(a, sizeof(Expression)); }

bool parse_int(Slice slice, int *out) {
    char buf[64];
    if (slice.len >= sizeof(buf)) { return false; }

    memcpy(buf, slice.ptr, slice.len);
    buf[slice.len] = '\0';

    char *end;
    long val = strtol(buf, &end, 10);

    if (end != buf + slice.len) { return false; }

    *out = (int)val;
    return true;
}

Type parse_type(Lexer *lexer) {
    Token type = lexer_next(lexer);

    switch (type.tag) {
    case TOK_KW_UNIT:
        return TYPE_UNIT;
    case TOK_KW_INT:
        return TYPE_INT;
    default:
        error_prefix(type.loc.start, lexer->source);
        fprintf(stderr, "Expected type but got `%s`.\n", token_tag_to_str(type.tag));
        exit(1);
    }
}

Block *parse_block_inner(Lexer *lexer, Arena *a) {
    Block *block = (Block *)arena_alloc(a, sizeof(Block));
    if (lexer_peek(lexer).tag == TOK_CURLY_CLOSE) {
        Expression *unit_expr = expr_init(a);
        unit_expr->tag = EXPR_UNIT;

        block->head = unit_expr;
        return block;
    };

    block->head = parse_expr(lexer, a);

    if (lexer_peek(lexer).tag == TOK_SEMICOLON) {
        lexer_next(lexer);
        block->tail = parse_block_inner(lexer, a);
    }

    return block;
}

ArgumentList *parse_args(Lexer *lexer, Arena *a, bool first) {
    if (lexer_peek(lexer).tag == TOK_PAREN_CLOSE) { return nullptr; }

    if (!first) {
        Token comma = lexer_next(lexer);
        expect(&comma, TOK_COMMA, lexer->source);

        if (lexer_peek(lexer).tag == TOK_PAREN_CLOSE) { return nullptr; }
    }

    Expression *e = parse_expr(lexer, a);

    ArgumentList *tail = parse_args(lexer, a, false);

    ArgumentList *args = (ArgumentList *)arena_alloc(a, sizeof(ArgumentList));
    args->head = e;
    args->tail = tail;
    return args;
}

VariableDefinition parse_vardef(Lexer *lexer, Arena *a, bool mutable) {
    Token variable = lexer_next(lexer);
    expect(&variable, TOK_IDENT, lexer->source);

    Token colon = lexer_next(lexer);
    expect(&colon, TOK_COLON, lexer->source);

    Token maybe_type = lexer_peek(lexer);
    TypeAnnotation annotation;
    if (maybe_type.tag == TOK_ASSIGN) {
        lexer_next(lexer);
        annotation.annotated = false;
    } else {
        annotation.type = parse_type(lexer);
        annotation.annotated = true;

        Token assign = lexer_next(lexer);
        expect(&assign, TOK_ASSIGN, lexer->source);
    }

    Expression *expr = parse_expr(lexer, a);

    return (VariableDefinition){
        .name = slice_from_location(lexer->source.buffer, variable.loc),
        .type_annotation = annotation,
        .expr = expr,
        .mutable = mutable,
    };
}

bool infix_bp(TokenTag op, size_t *l, size_t *r) {
    switch (op) {
    case TOK_ASSIGN:
        *l = 2;
        *r = 1;
        break;
    case TOK_PLUS:
        *l = 3;
        *r = 4;
        break;
    default:
        return false;
    }

    return true;
}

Expression *_parse_expr(Lexer *lexer, size_t min_bp, Arena *a) {
    Expression *e;

    Token tok = lexer_next(lexer);
    switch (tok.tag) {
    case TOK_KW_UNIT_EXPR:
        e = expr_init(a);
        e->tag = EXPR_UNIT;
        break;
    case TOK_LIT_INT:
        e = expr_init(a);
        e->tag = EXPR_INTEGER;
        if (!parse_int(slice_from_location(lexer->source.buffer, tok.loc), &e->integer)) {
            error_prefix(tok.loc.start, lexer->source);
            fprintf(stderr, "Parsing of integer failed");
            exit(1);
        }
        break;
    case TOK_IDENT:
        e = expr_init(a);
        if (lexer_peek(lexer).tag == TOK_PAREN_OPEN) {
            lexer_next(lexer);
            ArgumentList *args = parse_args(lexer, a, true);
            Token close_args = lexer_next(lexer);
            expect(&close_args, TOK_PAREN_CLOSE, lexer->source);

            e->tag = EXPR_FUNCALL;
            e->funcall = (FunctionCall){
                .function = slice_from_location(lexer->source.buffer, tok.loc),
                .args = args,
            };
        } else {
            e->tag = EXPR_VARIABLE;
            e->variable = slice_from_location(lexer->source.buffer, tok.loc);
        }
        break;

    case TOK_KW_VAL:
        e = expr_init(a);
        e->tag = EXPR_VARDEF;
        e->vardef = parse_vardef(lexer, a, false);
        break;

    case TOK_KW_VAR:
        e = expr_init(a);
        e->tag = EXPR_VARDEF;
        e->vardef = parse_vardef(lexer, a, true);
        break;

    case TOK_PAREN_OPEN:
        e = parse_expr(lexer, a);
        Token close = lexer_next(lexer);
        expect(&close, TOK_PAREN_CLOSE, lexer->source);
        break;
    case TOK_CURLY_OPEN:
        Block *block = parse_block_inner(lexer, a);
        Token close_block = lexer_next(lexer);
        expect(&close_block, TOK_CURLY_CLOSE, lexer->source);

        e = expr_init(a);
        e->tag = EXPR_BLOCK;
        e->block = block;
        break;
    default:
        error_prefix(tok.loc.start, lexer->source);
        fprintf(stderr, "Expected EXPRESSION but got `%s`.\n", token_tag_to_str(tok.tag));
        exit(1);
    }

    for (;;) {
        Token op = lexer_peek(lexer);
        if (op.tag == TOK_EOF) { break; }

        size_t l, r;
        if (infix_bp(op.tag, &l, &r)) {
            if (l < min_bp) { break; }
            lexer_next(lexer);

            Expression *rhs = _parse_expr(lexer, r, a);
            Expression *lhs = (Expression *)arena_alloc(a, sizeof(Expression));

            lhs->tag = EXPR_BINOP;
            lhs->binop.lhs = e;
            lhs->binop.rhs = rhs;

            switch (op.tag) {
            case TOK_PLUS:
                lhs->binop.kind = BINOP_ADD;
                break;
            case TOK_ASSIGN:
                lhs->binop.kind = BINOP_ASSIGN;
                break;
            default:
                fprintf(stderr, "unreachable, _parse_expr, infix");
                exit(1);
                break;
            }

            e = lhs;
            continue;
        }

        break;
    }

    return e;
}

Expression *parse_expr(Lexer *lexer, Arena *a) { return _parse_expr(lexer, 0, a); }

ParameterList *parse_params(Lexer *lexer, Arena *a, bool first) {
    if (lexer_peek(lexer).tag == TOK_PAREN_CLOSE) { return nullptr; }

    if (!first) {
        Token comma = lexer_next(lexer);
        expect(&comma, TOK_COMMA, lexer->source);

        if (lexer_peek(lexer).tag == TOK_PAREN_CLOSE) { return nullptr; }
    }

    Parameter param = {0};

    Token maybe_var = lexer_peek(lexer);
    if (maybe_var.tag == TOK_KW_VAR) {
        param.mutable = true;
        lexer_next(lexer);
    } else if (maybe_var.tag == TOK_KW_VAL) {
        lexer_next(lexer);
    }

    Token param_name = lexer_next(lexer);
    expect(&param_name, TOK_IDENT, lexer->source);
    param.name = slice_from_location(lexer->source.buffer, param_name.loc);

    Token colon = lexer_next(lexer);
    expect(&colon, TOK_COLON, lexer->source);

    param.type = parse_type(lexer);

    ParameterList *tail = parse_params(lexer, a, false);

    ParameterList *params = (ParameterList *)arena_alloc(a, sizeof(ParameterList));
    params->head = param;
    params->tail = tail;
    return params;
}

Function parse_function(Lexer *lexer, Arena *a) {
    Function f = {0};

    Token fun = lexer_next(lexer);
    expect(&fun, TOK_KW_FUN, lexer->source);

    Token name = lexer_next(lexer);
    expect(&name, TOK_IDENT, lexer->source);
    f.name = slice_from_location(lexer->source.buffer, name.loc);

    Token open = lexer_next(lexer);
    expect(&open, TOK_PAREN_OPEN, lexer->source);

    f.params = parse_params(lexer, a, true);

    Token close = lexer_next(lexer);
    expect(&close, TOK_PAREN_CLOSE, lexer->source);

    Token colon = lexer_next(lexer);
    expect(&colon, TOK_COLON, lexer->source);

    f.return_type = parse_type(lexer);

    Token assign = lexer_next(lexer);
    expect(&assign, TOK_ASSIGN, lexer->source);

    f.body = parse_expr(lexer, a);

    return f;
}

Program *parse_program(Lexer *lexer, Arena *a) {
    if (lexer_peek(lexer).tag == TOK_EOF) { return nullptr; }

    Function f = parse_function(lexer, a);
    Program *tail = parse_program(lexer, a);

    Program *p = (Program *)arena_alloc(a, sizeof(Program));
    p->head = f;
    p->tail = tail;

    return p;
}
