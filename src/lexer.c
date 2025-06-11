/* LEXER
 * -----
 * The lexer is based on Zig's tokenizer:
 * https://github.com/ziglang/zig/blob/master/lib/std/zig/tokenizer.zig
 *
 */

#include "lexer.h"

#include <stddef.h>

#include "util/slice.h"
#include "util/source.h"

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
        return "fun";
    case TOK_KW_INT:
        return "Int";
    case TOK_KW_UNIT:
        return "Unit";
    case TOK_KW_UNIT_EXPR:
        return "unit";
    case TOK_KW_VAL:
        return "val";
    case TOK_KW_VAR:
        return "var";
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

bool is_alpha(char c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }
bool is_digit(char c) { return c >= '0' && c <= '9'; }

Lexer lexer_init(SourceContext source) {
    return (Lexer){
        .source = source,

        /* NOTE: we may want to skip a potential UTF-8 BOM here */
        .index = 0,
    };
}

char lexer_current(Lexer *lexer) { return lexer->source.buffer[lexer->index]; }

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
    result.loc.start = lexer->index;

    State state = STATE_START;
    char c = lexer_current(lexer);
    for (;;) {
        switch (state) {
        case STATE_START:
            c = lexer_current(lexer);
            switch (c) {
            case 0:
                if (lexer->index == lexer->source.len) {
                    return (Token){
                        .tag = TOK_EOF,
                        .loc = {.start = lexer->index, .end = lexer->index},
                    };
                }

                state = STATE_INVALID;
                continue;

            case ' ':
            case '\n':
            case '\t':
            case '\r':
                lexer->index++;
                result.loc.start = lexer->index;

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
                c = lexer_current(lexer);
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
            c = lexer_current(lexer);
            if (is_alpha(c) || is_digit(c) || c == '_') {
                state = STATE_IDENT;
                continue;
            }

            size_t ident_len = lexer->index - result.loc.start;
            Slice ident = slice(lexer->source.buffer, result.loc.start, ident_len);
            token_tag_keyword_from_slice(ident, &result.tag);
            break;

        case STATE_INT:
            c = lexer_current(lexer);
            if (c == '_' || is_digit(c)) {
                lexer->index++;

                state = STATE_INT;
                continue;
            }
            break;

        case STATE_SLASH:
            lexer->index++;
            c = lexer_current(lexer);
            if (c == '/') {
                state = STATE_COMMENT;
                continue;
            }

            result.tag = TOK_INVALID; /* TODO: Replace with TOK_SLASH */
            break;

        case STATE_COMMENT:
            lexer->index++;
            c = lexer_current(lexer);
            if (c == '\n') {
                lexer->index++;
                result.loc.start = lexer->index;
                state = STATE_START;
                continue;
            }

            state = STATE_COMMENT;
            continue;

        case STATE_INVALID:
            lexer->index++;

            c = lexer_current(lexer);
            if ((c == 0 && lexer->index == lexer->source.len) || c == '\n') {
                result.tag = TOK_INVALID;
                break;
            }

            state = STATE_INVALID;
            continue;
        }

        break;
    }

    result.loc.end = lexer->index;
    return result;
}

Token lexer_peek(Lexer *lexer) {
    size_t index = lexer->index;
    Token peek = lexer_next(lexer);
    lexer->index = index;
    return peek;
}
