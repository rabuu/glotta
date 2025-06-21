/* PARSER
 * ------
 * Pratt parsing: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
 */

#include "parsing.h"

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ast.h"
#include "lexing.h"
#include "source.h"
#include "util/arena.h"
#include "util/slice.h"

typedef struct {
    Lexer *l;
    Arena *arena;
    Arena *tmp;

    Expression *unit_expr;
} Parser;

/* forward declarations */
static Expression *parse_expr(Parser *p);

static void error_prefix(size_t index, SourceContext source) {
    FilePosition fpos = file_position(index, source);
    fprintf(stderr, "ERROR[%s:%zu:%zu]: ", source.filename, fpos.row, fpos.column);
}

static void expect(Token *token, TokenTag tag, SourceContext source) {
    if (token->tag != tag) {
        char *expected = token_tag_to_str(tag);
        char *but_got = token_tag_to_str(token->tag);

        error_prefix(token->loc.start, source);
        fprintf(stderr, "Expected `%s` but got `%s`.\n", expected, but_got);
        exit(1);
    }
}

static Expression *expr_init(Arena *a) { return arena_alloc(a, sizeof(Expression)); }

static bool parse_int(StrSlice slice, int *out) {
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

static Type parse_type(Parser *p) {
    Token type = lexer_next(p->l);

    switch (type.tag) {
    case TOK_KW_UNIT:
        return TYPE_UNIT;
    case TOK_KW_INT:
        return TYPE_INT;
    default:
        error_prefix(type.loc.start, p->l->source);
        fprintf(stderr, "Expected type but got `%s`.\n", token_tag_to_str(type.tag));
        exit(1);
    }
}

typedef struct ExprList {
    Expression *head;
    struct ExprList *tail;
} ExprList;

static size_t exprlist_count(ExprList *list) {
    if (!list) { return 0; }
    return 1 + exprlist_count(list->tail);
}

static ExprList *_parse_block(Parser *p) {
    ExprList *exprs = (ExprList *)arena_alloc(p->tmp, sizeof(ExprList));
    if (lexer_peek(p->l).tag == TOK_CURLY_CLOSE) {
        exprs->head = p->unit_expr;
        return exprs;
    };

    exprs->head = parse_expr(p);

    if (lexer_peek(p->l).tag == TOK_SEMICOLON) {
        lexer_next(p->l);
        exprs->tail = _parse_block(p);
    }

    return exprs;
}

static Block parse_block(Parser *p) {
    ExprList *exprs = _parse_block(p);

    size_t len = exprlist_count(exprs);
    Expression **items = arena_alloc(p->arena, len * sizeof(Expression *));

    for (size_t i = 0; i < len; ++i) {
        items[i] = exprs->head;
        exprs = exprs->tail;
    }

    Block block = {
        .items = items,
        .len = len,
    };
    return block;
}

static ExprList *_parse_args(Parser *p, bool first) {
    if (lexer_peek(p->l).tag == TOK_PAREN_CLOSE) { return nullptr; }

    if (!first) {
        Token comma = lexer_next(p->l);
        expect(&comma, TOK_COMMA, p->l->source);
        if (lexer_peek(p->l).tag == TOK_PAREN_CLOSE) { return nullptr; }
    }

    Expression *e = parse_expr(p);
    ExprList *tail = _parse_args(p, false);

    ExprList *args = (ExprList *)arena_alloc(p->tmp, sizeof(ExprList));
    args->head = e;
    args->tail = tail;
    return args;
}

static Arguments parse_args(Parser *p) {
    ExprList *args = _parse_args(p, true);

    size_t len = exprlist_count(args);
    Expression **items = arena_alloc(p->arena, len * sizeof(Expression *));

    for (size_t i = 0; i < len; ++i) {
        items[i] = args->head;
        args = args->tail;
    }

    Arguments arguments = {
        .items = items,
        .len = len,
    };
    return arguments;
}

static VariableDefinition parse_vardef(Parser *p, bool mutable) {
    Token variable = lexer_next(p->l);
    expect(&variable, TOK_IDENT, p->l->source);

    Token colon = lexer_next(p->l);
    expect(&colon, TOK_COLON, p->l->source);

    Token maybe_type = lexer_peek(p->l);
    TypeAnnotation annotation;
    if (maybe_type.tag == TOK_ASSIGN) {
        lexer_next(p->l);
        annotation.annotated = false;
    } else {
        annotation.type = parse_type(p);
        annotation.pos = file_position(maybe_type.loc.start, p->l->source);
        annotation.annotated = true;

        Token assign = lexer_next(p->l);
        expect(&assign, TOK_ASSIGN, p->l->source);
    }

    Expression *expr = parse_expr(p);

    return (VariableDefinition){
        .name = strslice_from_loc(p->l->source.buffer, variable.loc),
        .type_annotation = annotation,
        .expr = expr,
        .mutable = mutable,
    };
}

static bool op_infix(TokenTag op, size_t *l, size_t *r) {
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

static Expression *_parse_expr(Parser *p, size_t min_bp) {
    Expression *e;

    Token tok = lexer_next(p->l);
    switch (tok.tag) {
    case TOK_KW_UNIT_EXPR:
        e = p->unit_expr;
        break;
    case TOK_LIT_INT:
        e = expr_init(p->arena);
        e->tag = EXPR_INTEGER;
        if (!parse_int(strslice_from_loc(p->l->source.buffer, tok.loc), &e->integer)) {
            error_prefix(tok.loc.start, p->l->source);
            fprintf(stderr, "Parsing of integer failed");
            exit(1);
        }
        break;
    case TOK_IDENT:
        e = expr_init(p->arena);
        if (lexer_peek(p->l).tag == TOK_PAREN_OPEN) {
            lexer_next(p->l);
            Arguments args = parse_args(p);
            Token close_args = lexer_next(p->l);
            expect(&close_args, TOK_PAREN_CLOSE, p->l->source);

            e->tag = EXPR_FUNCALL;
            e->funcall = (FunctionCall){
                .function = strslice_from_loc(p->l->source.buffer, tok.loc),
                .args = args,
            };
        } else {
            e->tag = EXPR_VARIABLE;
            e->variable.name = strslice_from_loc(p->l->source.buffer, tok.loc);
        }
        break;

    case TOK_KW_VAL:
        e = expr_init(p->arena);
        e->tag = EXPR_VARDEF;
        e->vardef = parse_vardef(p, false);
        break;

    case TOK_KW_VAR:
        e = expr_init(p->arena);
        e->tag = EXPR_VARDEF;
        e->vardef = parse_vardef(p, true);
        break;

    case TOK_PAREN_OPEN:
        e = parse_expr(p);
        Token close = lexer_next(p->l);
        expect(&close, TOK_PAREN_CLOSE, p->l->source);
        break;
    case TOK_CURLY_OPEN:
        Block block = parse_block(p);
        Token close_block = lexer_next(p->l);
        expect(&close_block, TOK_CURLY_CLOSE, p->l->source);

        e = expr_init(p->arena);
        e->tag = EXPR_BLOCK;
        e->block = block;
        break;
    default:
        error_prefix(tok.loc.start, p->l->source);
        fprintf(stderr, "Expected EXPRESSION but got `%s`.\n", token_tag_to_str(tok.tag));
        exit(1);
    }

    e->pos = file_position(tok.loc.start, p->l->source);

    for (;;) {
        Token op = lexer_peek(p->l);
        if (op.tag == TOK_EOF) { break; }

        size_t l, r;
        if (op_infix(op.tag, &l, &r)) {
            if (l < min_bp) { break; }
            lexer_next(p->l);

            Expression *rhs = _parse_expr(p, r);
            Expression *lhs = expr_init(p->arena);

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

static Expression *parse_expr(Parser *p) { return _parse_expr(p, 0); }

typedef struct ParamList {
    Parameter head;
    struct ParamList *tail;
} ParamList;

static size_t paramlist_count(ParamList *list) {
    if (!list) { return 0; }
    return 1 + paramlist_count(list->tail);
}

static ParamList *_parse_params(Parser *p, bool first) {
    if (lexer_peek(p->l).tag == TOK_PAREN_CLOSE) { return nullptr; }

    if (!first) {
        Token comma = lexer_next(p->l);
        expect(&comma, TOK_COMMA, p->l->source);

        if (lexer_peek(p->l).tag == TOK_PAREN_CLOSE) { return nullptr; }
    }

    Parameter param = {0};

    Token maybe_var = lexer_peek(p->l);
    if (maybe_var.tag == TOK_KW_VAR) {
        param.mutable = true;
        lexer_next(p->l);
    } else if (maybe_var.tag == TOK_KW_VAL) {
        lexer_next(p->l);
    }

    Token param_name = lexer_next(p->l);
    expect(&param_name, TOK_IDENT, p->l->source);
    param.name = strslice_from_loc(p->l->source.buffer, param_name.loc);

    Token colon = lexer_next(p->l);
    expect(&colon, TOK_COLON, p->l->source);

    param.type_annotation.pos = file_position(p->l->index, p->l->source);
    param.type_annotation.type = parse_type(p);
    param.type_annotation.annotated = true;

    param.pos = file_position(param_name.loc.start, p->l->source);

    ParamList *tail = _parse_params(p, false);

    ParamList *params = (ParamList *)arena_alloc(p->tmp, sizeof(ParamList));
    params->head = param;
    params->tail = tail;
    return params;
}

static Parameters parse_params(Parser *p) {
    ParamList *params = _parse_params(p, true);

    size_t len = paramlist_count(params);
    Parameter *items = arena_alloc(p->arena, len * sizeof(Parameter));

    for (size_t i = 0; i < len; ++i) {
        items[i] = params->head;
        params = params->tail;
    }

    Parameters parameters = {
        .items = items,
        .len = len,
    };
    return parameters;
}

static Function parse_function(Parser *p) {
    Function f = {0};

    Token fun = lexer_next(p->l);
    expect(&fun, TOK_KW_FUN, p->l->source);

    Token name = lexer_next(p->l);
    expect(&name, TOK_IDENT, p->l->source);
    f.name = strslice_from_loc(p->l->source.buffer, name.loc);

    Token open = lexer_next(p->l);
    expect(&open, TOK_PAREN_OPEN, p->l->source);

    f.params = parse_params(p);

    Token close = lexer_next(p->l);
    expect(&close, TOK_PAREN_CLOSE, p->l->source);

    if (lexer_peek(p->l).tag == TOK_COLON) {
        lexer_next(p->l);
        f.return_type_annotation.pos = file_position(p->l->index, p->l->source);
        f.return_type_annotation.type = parse_type(p);
        f.return_type_annotation.annotated = true;
    } else {
        f.return_type_annotation.annotated = false;
    }

    Token assign = lexer_next(p->l);
    expect(&assign, TOK_ASSIGN, p->l->source);

    f.body = parse_expr(p);

    f.pos = file_position(fun.loc.start, p->l->source);

    return f;
}

typedef struct FunList {
    Function head;
    struct FunList *tail;
} FunList;

static size_t funlist_count(FunList *list) {
    if (!list) { return 0; }
    return 1 + funlist_count(list->tail);
}

static FunList *_parse_program(Parser *p) {
    if (lexer_peek(p->l).tag == TOK_EOF) { return nullptr; }

    Function f = parse_function(p);
    FunList *tail = _parse_program(p);

    FunList *funs = arena_alloc(p->tmp, sizeof(FunList));
    funs->head = f;
    funs->tail = tail;

    return funs;
}

Program parse_program(Lexer *lexer, Arena *ast_arena) {
    Arena tmp = {0};

    Expression *unit_expr = expr_init(ast_arena);
    unit_expr->tag = EXPR_UNIT;

    Parser p = {
        .l = lexer,
        .arena = ast_arena,
        .tmp = &tmp,
        .unit_expr = unit_expr,
    };

    FunList *funs = _parse_program(&p);
    size_t function_count = funlist_count(funs);

    Function *functions = arena_alloc(ast_arena, function_count * sizeof(Function));
    for (size_t i = 0; i < function_count; ++i) {
        functions[i] = funs->head;
        funs = funs->tail;
    }

    Program program = {
        .functions = functions,
        .function_count = function_count,
    };

    arena_free(&tmp);

    return program;
}
