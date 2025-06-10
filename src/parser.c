/* PARSER
 * ------
 * Pratt parsing: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
 */

#include "parser.h"

#include <stdio.h>
#include <stdlib.h>

#include "ast.h"
#include "lexer.h"
#include "util/arena.h"
#include "util/slice.h"
#include "util/source.h"

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

bool infix_bp(TokenTag op, size_t *l, size_t *r) {
    switch (op) {
    case TOK_PLUS:
        *l = 1;
        *r = 2;
        break;
    default:
        return false;
    }

    return true;
}

Block *parse_block_inner(Lexer *lexer, Arena *a) {
    Block *block = nullptr;
    if (lexer_peek(lexer).tag == TOK_CURLY_CLOSE) { return block; };

    block = (Block *)arena_alloc(a, sizeof(Block));

    block->head = parse_expr(lexer, a);

    if (lexer_peek(lexer).tag == TOK_SEMICOLON) {
        lexer_next(lexer);
        block->tail = parse_block_inner(lexer, a);
    }

    return block;
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
        e->integer = 0; /* FIXME: assign correct value */
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

Function parse_function(Lexer *lexer, Arena *a) {
    Function f = {0};

    Token fun = lexer_next(lexer);
    expect(&fun, TOK_KW_FUN, lexer->source);

    Token name = lexer_next(lexer);
    expect(&name, TOK_IDENT, lexer->source);
    f.name = slice_from_location(lexer->source.buffer, name.loc);

    Token open = lexer_next(lexer);
    expect(&open, TOK_PAREN_OPEN, lexer->source);

    ParameterList *params = nullptr;

    bool first_param = true;
    for (;;) {
        Token peek = lexer_peek(lexer);
        if (peek.tag == TOK_PAREN_CLOSE) { break; }

        if (!first_param) {
            Token comma = lexer_next(lexer);
            expect(&comma, TOK_COMMA, lexer->source);
        }
        first_param = false;

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

        ParameterList *new_params = (ParameterList *)arena_alloc(a, sizeof(ParameterList));
        new_params->head = param;
        new_params->tail = params;
        params = new_params;
    }
    f.params = params;

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
