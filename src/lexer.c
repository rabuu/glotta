/* LEXER
 * -----
 * The lexer is based on Zig's tokenizier:
 * https://github.com/ziglang/zig/blob/master/lib/std/zig/tokenizer.zig
 *
 */

#include "lexer.h"
#include "util.h"

#include <stddef.h>
#include <stdio.h>
#include <string.h>

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
    STATE_INVALID,
} State;

bool is_alpha(char c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }

bool is_digit(char c) { return c >= '0' && c <= '9'; }

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
            if (slice_eq_str(&ident, "fn")) {
                result.tag = TOK_KW_FN;
            } else if (slice_eq_str(&ident, "Int")) {
                result.tag = TOK_KW_INT;
            }
            break;

        case STATE_INT:
            c = lexer->buffer[lexer->index];
            if (c == '_' || is_digit(c)) {
                lexer->index++;

                state = STATE_INT;
                continue;
            }
            break;

        case STATE_INVALID:
            lexer->index++;
            c = lexer->buffer[lexer->index];
            if (c == 0) {
                if (lexer->index == lexer->len) {
                    result.tag = TOK_INVALID;
                }
                state = STATE_INVALID;
                continue;
            } else if (c == '\n') {
                result.tag = TOK_INVALID;
            }

            state = STATE_INVALID;
            continue;
        }

        break;
    }

    result.pos.end = lexer->index;
    return result;
}

char *token_tag_to_string(TokenTag tag) {
    switch (tag) {
    case TOK_KW_FN:
        return "fn";
    case TOK_KW_INT:
        return "Int";
    case TOK_LIT_INT:
        return "INTEGER";
    case TOK_IDENT:
        return "IDENTIFIER";
    case TOK_PAREN_OPEN:
        return "(";
    case TOK_PAREN_CLOSE:
        return ")";
    case TOK_CURLY_OPEN:
        return "{";
    case TOK_CURLY_CLOSE:
        return "}";
    case TOK_COMMA:
        return ",";
    case TOK_COLON:
        return ":";
    case TOK_SEMICOLON:
        return ";";
    case TOK_ASSIGN:
        return "=";
    case TOK_PLUS:
        return "+";
    case TOK_EOF:
        return "EOF";
    case TOK_INVALID:
        return "INVALID";
    }
}

char* debug_token(Token *token) {
    return token_tag_to_string(token->tag);
}
