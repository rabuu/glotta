#ifndef LEXER_H_
#define LEXER_H_

#include <stddef.h>

#include "util.h"

typedef struct {
    char *buffer;
    const size_t len;

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
    SourcePosition pos;
} Token;

Lexer lexer_init(char *buffer);
Token lexer_next(Lexer *lexer);

void debug_token(Token *token, char *source);

#endif // LEXER_H_
