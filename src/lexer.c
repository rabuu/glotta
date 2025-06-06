/* LEXER
 * -----
 * The lexer is based on Zig's tokenizer:
 * https://github.com/ziglang/zig/blob/master/lib/std/zig/tokenizer.zig
 *
 */

#include "lexer.h"

#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "util.h"

/**
 * If is `str` is a valid keyword, set `tag` to corresponding value.
 * Otherwise, do nothing.
 *
 * Return `true` if `tag` was changed, `false` otherwise.
 */
bool token_tag_keyword_from_slice(Slice str, TokenTag *tag) {
    if (slice_eq_str(str, "fun")) {
        *tag = TOK_KW_FUN;
    } else if (slice_eq_str(str, "Int")) {
        *tag = TOK_KW_INT;
    } else if (slice_eq_str(str, "Unit")) {
        *tag = TOK_KW_UNIT;
    } else if (slice_eq_str(str, "unit")) {
        *tag = TOK_KW_UNIT_EXPR;
    } else if (slice_eq_str(str, "val")) {
        *tag = TOK_KW_VAL;
    } else if (slice_eq_str(str, "var")) {
        *tag = TOK_KW_VAR;
    } else {
        return false;
    }

    return true;
}

char *token_tag_to_str(TokenTag tag) {
    switch (tag) {
    case TOK_KW_FUN:
    case TOK_KW_INT:
    case TOK_KW_UNIT:
    case TOK_KW_UNIT_EXPR:
    case TOK_KW_VAL:
    case TOK_KW_VAR:
        return "KEYWORD";
    case TOK_LIT_INT:
        return "INTEGER";
    case TOK_IDENT:
        return "IDENTIFIER";
    case TOK_PAREN_OPEN:
        return "PAREN_OPEN";
    case TOK_PAREN_CLOSE:
        return "PAREN_CLOSE";
    case TOK_CURLY_OPEN:
        return "CURLY_OPEN";
    case TOK_CURLY_CLOSE:
        return "CURLY_CLOSE";
    case TOK_COMMA:
        return "COMMA";
    case TOK_COLON:
        return "COLON";
    case TOK_SEMICOLON:
        return "SEMICOLON";
    case TOK_ASSIGN:
        return "ASSIGN";
    case TOK_PLUS:
        return "PLUS";
    case TOK_EOF:
        return "EOF";
    case TOK_INVALID:
        return "INVALID";
    }
}

bool is_alpha(char c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }
bool is_digit(char c) { return c >= '0' && c <= '9'; }

Lexer lexer_init(char *buffer) {
    size_t len = strlen(buffer);

    return (Lexer){
        .buffer = buffer,
        .len = len,

        /* NOTE: we may want to skip a potential UTF-8 BOM here */
        .index = 0,
    };
}

typedef enum {
    STATE_START,
    STATE_IDENT,
    STATE_INT,
    STATE_SLASH,
    STATE_COMMENT,
    STATE_INVALID,
} State;

Token lexer_next(Lexer *lexer) {
    Token result;
    result.pos.start = lexer->index;

    State state = STATE_START;
    char c = lexer->buffer[lexer->index];
    for (;;) {
        switch (state) {
        case STATE_START:
            c = lexer->buffer[lexer->index];
            switch (c) {
            case 0:
                if (lexer->index == lexer->len) {
                    return (Token){
                        .tag = TOK_EOF,
                        .pos = {.start = lexer->index, .end = lexer->index},
                    };
                }

                state = STATE_INVALID;
                continue;

            case ' ':
            case '\n':
            case '\t':
            case '\r':
                lexer->index++;
                result.pos.start = lexer->index;

                state = STATE_START;
                continue;

            case '/':
                state = STATE_SLASH;
                continue;

            case '(':
                result.tag = TOK_PAREN_OPEN;
                lexer->index++;
                break;
            case ')':
                result.tag = TOK_PAREN_CLOSE;
                lexer->index++;
                break;
            case '{':
                result.tag = TOK_CURLY_OPEN;
                lexer->index++;
                break;
            case '}':
                result.tag = TOK_CURLY_CLOSE;
                lexer->index++;
                break;
            case ',':
                result.tag = TOK_COMMA;
                lexer->index++;
                break;
            case ':':
                result.tag = TOK_COLON;
                lexer->index++;
                break;
            case ';':
                result.tag = TOK_SEMICOLON;
                lexer->index++;
                break;
            case '=':
                result.tag = TOK_ASSIGN;
                lexer->index++;
                break;
            case '+':
                result.tag = TOK_PLUS;
                lexer->index++;
                break;

            default:
                c = lexer->buffer[lexer->index];
                if (is_alpha(c) || c == '_') {
                    result.tag = TOK_IDENT;

                    state = STATE_IDENT;
                    continue;
                } else if (is_digit(c)) {
                    result.tag = TOK_LIT_INT;

                    state = STATE_INT;
                    continue;
                }

                state = STATE_INVALID;
                continue;
            }
            break;

        case STATE_IDENT:
            lexer->index++;
            c = lexer->buffer[lexer->index];
            if (is_alpha(c) || is_digit(c) || c == '_') {
                state = STATE_IDENT;
                continue;
            }

            Slice ident = slice(lexer->buffer, result.pos.start, lexer->index - result.pos.start);
            token_tag_keyword_from_slice(ident, &result.tag);
            break;

        case STATE_INT:
            c = lexer->buffer[lexer->index];
            if (c == '_' || is_digit(c)) {
                lexer->index++;

                state = STATE_INT;
                continue;
            }
            break;

        case STATE_SLASH:
            lexer->index++;
            c = lexer->buffer[lexer->index];
            if (c == '/') {
                state = STATE_COMMENT;
                continue;
            }

            result.tag = TOK_INVALID; /* TODO: Replace with TOK_SLASH */
            break;

        case STATE_COMMENT:
            lexer->index++;
            c = lexer->buffer[lexer->index];
            if (c == '\n') {
                lexer->index++;
                result.pos.start = lexer->index;
                state = STATE_START;
                continue;
            }

            state = STATE_COMMENT;
            continue;

        case STATE_INVALID:
            lexer->index++;

            c = lexer->buffer[lexer->index];
            if ((c == 0 && lexer->index == lexer->len) || c == '\n') {
                result.tag = TOK_INVALID;
                break;
            }

            state = STATE_INVALID;
            continue;
        }

        break;
    }

    result.pos.end = lexer->index;
    return result;
}

void debug_token(Token *token, char *source) {
    char *tag = token_tag_to_str(token->tag);
    Slice lexeme = slice_from_source(source, token->pos);

    printf("%s", tag);
    if (lexeme.len > 0) {
        printf(": ");
        slice_print(lexeme);
    }
    printf("\n");
}
