/* PARSER
 * ------
 * Pratt parsing: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
 */

#include <stdio.h>
#include <stdlib.h>

#include "ast.h"
#include "lexer.h"
#include "util/arena.h"
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

bool infix_bp(TokenTag op, size_t *l, size_t *r) {
    switch (op) {
    case TOK_PLUS:
        *l = 5;
        *r = 6;
        break;
    default:
        return false;
    }

    return true;
}

Expression *pratt_expr(Lexer *lexer, size_t min_bp, Arena *a) {
    Expression *lhs;

    Token tok = lexer_next(lexer);
    switch (tok.tag) {
    case TOK_LIT_INT:
        lhs = (Expression *)arena_alloc(a, sizeof(Expression));
        lhs->tag = EXPR_INTEGER;
        lhs->integer = 0;
        break;
    case TOK_PAREN_OPEN:
        lhs = pratt_expr(lexer, 0, a);
        Token close = lexer_next(lexer);
        expect(&close, TOK_PAREN_CLOSE, lexer->source);
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

            Expression *rhs = pratt_expr(lexer, r, a);
            Expression *new_lhs = (Expression *)arena_alloc(a, sizeof(Expression));

            new_lhs->tag = EXPR_BINOP;
            new_lhs->binop.lhs = lhs;
            new_lhs->binop.rhs = rhs;

            switch (op.tag) {
            case TOK_PLUS:
                new_lhs->binop.kind = BINOP_ADD;
                break;
            default:
                break;
            }

            lhs = new_lhs;
            continue;
        }

        break;
    }

    return lhs;
}

Expression *parse_expr(Lexer *lexer, Arena *a) { return pratt_expr(lexer, 0, a); }

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

void parse_function(Lexer *lexer, Arena *a) {
    Token fun = lexer_next(lexer);
    expect(&fun, TOK_KW_FUN, lexer->source);

    Token name = lexer_next(lexer);
    expect(&name, TOK_IDENT, lexer->source);

    Token open = lexer_next(lexer);
    expect(&open, TOK_PAREN_OPEN, lexer->source);

    bool first_param = true;
    for (;;) {
        Token peek = lexer_peek(lexer);
        if (peek.tag == TOK_PAREN_CLOSE) { break; }

        if (!first_param) {
            Token comma = lexer_next(lexer);
            expect(&comma, TOK_COMMA, lexer->source);
        }
        first_param = false;

        Token param_name = lexer_next(lexer);
        expect(&param_name, TOK_IDENT, lexer->source);

        Token colon = lexer_next(lexer);
        expect(&colon, TOK_COLON, lexer->source);

        parse_type(lexer);
    }

    Token close = lexer_next(lexer);
    expect(&close, TOK_PAREN_CLOSE, lexer->source);

    Token colon = lexer_next(lexer);
    expect(&colon, TOK_COLON, lexer->source);

    parse_type(lexer);

    Token assign = lexer_next(lexer);
    expect(&assign, TOK_ASSIGN, lexer->source);

    Expression *body = parse_expr(lexer, a);
}
