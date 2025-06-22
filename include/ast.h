#ifndef AST_H_
#define AST_H_

#include <stdint.h>

#include "source.h"
#include "util/slice.h"

typedef size_t SymbolId;

typedef struct AstExpr AstExpr;

typedef enum {
    TYPE_UNIT,
    TYPE_INT,
} AstType;

typedef struct {
    bool annotated;
    AstType type;
    FilePosition pos;
} AstTypeAnnotation;

typedef struct {
    bool resolved;
    AstType type;
} AstInferredType;

typedef struct {
    StrSlice name;
    SymbolId symbol;
} AstVariable;

typedef struct {
    enum {
        BINOP_ASSIGN,
        BINOP_ADD,
    } kind;
    AstExpr *lhs;
    AstExpr *rhs;
} AstBinaryOp;

typedef struct {
    AstExpr **items;
    size_t len;
} AstArguments;

typedef struct {
    StrSlice function;
    SymbolId symbol;
    AstArguments args;
} AstFunCall;

typedef struct {
    StrSlice name;
    SymbolId symbol;
    AstTypeAnnotation type_annotation;
    AstExpr *expr;
    bool mutable;
} AstVarDef;

typedef struct {
    AstExpr **items;
    size_t len;
} AstBlock;

struct AstExpr {
    enum {
        EXPR_UNIT,
        EXPR_INTEGER,
        EXPR_VARIABLE,
        EXPR_BINOP,
        EXPR_FUNCALL,
        EXPR_VARDEF,
        EXPR_BLOCK,
    } tag;
    union {
        int32_t integer;
        AstVariable variable;
        AstBinaryOp binop;
        AstFunCall funcall;
        AstVarDef vardef;
        AstBlock block;
    };
    FilePosition pos;
    AstInferredType inferred_type;
};

typedef struct {
    StrSlice name;
    SymbolId symbol;
    AstTypeAnnotation type_annotation;
    bool mutable;
    FilePosition pos;
} AstParam;

typedef struct {
    AstParam *items;
    size_t len;
} AstParameters;

typedef struct {
    StrSlice name;
    SymbolId symbol;
    AstParameters params;
    AstTypeAnnotation return_type_annotation;
    AstExpr *body;
    FilePosition pos;
} AstFunction;

typedef struct {
    AstFunction *functions;
    size_t function_count;
} AstProgram;

#endif // AST_H_
