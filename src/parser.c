/* PARSER
 * ------
 * Pratt parsing: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
 */

#include <stdio.h>
#include <stdlib.h>

#include "ast.h"
#include "lexer.h"
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

void parse_function(Lexer *lexer) {
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
}
