#ifndef LEXER_H_
#define LEXER_H_

#include <stddef.h>

#include "source.h"

typedef struct {
    SourceContext source;
    size_t index;
} Lexer;

typedef enum {
    TOK_KW_FUN,
    TOK_KW_INT,
    TOK_KW_UNIT,
    TOK_KW_UNIT_EXPR,
    TOK_KW_VAL,
    TOK_KW_VAR,

    TOK_LIT_INT,

    TOK_IDENT,

    TOK_PAREN_OPEN,
    TOK_PAREN_CLOSE,
    TOK_CURLY_OPEN,
    TOK_CURLY_CLOSE,

    TOK_DOT,
    TOK_COMMA,
    TOK_COLON,
    TOK_SEMICOLON,
    TOK_ASSIGN,
    TOK_PLUS,

    TOK_EOF,
    TOK_INVALID,
} TokenTag;

typedef struct {
    TokenTag tag;
    Location loc;
} Token;

Lexer lexer_init(SourceContext source);
char lexer_current(Lexer *lexer);
Token lexer_next(Lexer *lexer);
Token lexer_peek(Lexer *lexer);

char *token_tag_to_str(TokenTag tag);

#endif // LEXER_H_
