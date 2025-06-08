/* PARSER
 * ------
 * Pratt parsing: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
 */

#include <stdio.h>
#include <stdlib.h>

#include "lexer.h"
#include "ast.h"

void expect(Token *token, TokenTag tag) {
    if (token->tag != tag) {
        char *expected = token_tag_to_str(tag);
        char *but_got = token_tag_to_str(token->tag);
        fprintf(stderr, "ERROR: Expected `%s` but got `%s`.\n", expected, but_got);
        exit(1);
    }
}

Type parse_type(Lexer *lexer) {
    Token type = lexer_next(lexer);

    switch (type.tag) {
        case TOK_KW_UNIT:
            return TYPE_UNIT;
        case TOK_KW_INT:
            return TYPE_INT;
        default:
            fprintf(stderr, "ERROR: Expected type but got `%s`.\n", token_tag_to_str(type.tag));
            exit(1);
    }
}

void parse_function(Lexer *lexer) {
    Token fun = lexer_next(lexer);
    expect(&fun, TOK_KW_FUN);

    Token name = lexer_next(lexer);
    expect(&name, TOK_IDENT);

    Token open = lexer_next(lexer);
    expect(&open, TOK_PAREN_OPEN);

    bool first_param = true;
    for (;;) {
        Token peek = lexer_peek(lexer);
        if (peek.tag == TOK_PAREN_CLOSE) { break; }

        if (!first_param) {
            Token comma = lexer_next(lexer);
            expect(&comma, TOK_COMMA);
        }
        first_param = false;

        Token param_name = lexer_next(lexer);
        expect(&param_name, TOK_IDENT);

        Token colon = lexer_next(lexer);
        expect(&colon, TOK_COLON);

        parse_type(lexer);
    }

    Token close = lexer_next(lexer);
    expect(&close, TOK_PAREN_CLOSE);

    Token colon = lexer_next(lexer);
    expect(&colon, TOK_COLON);

    parse_type(lexer);

    Token assign = lexer_next(lexer);
    expect(&assign, TOK_ASSIGN);
}
