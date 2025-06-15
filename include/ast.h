#ifndef AST_H_
#define AST_H_

#include <stdint.h>

#include "source.h"
#include "util/slice.h"

typedef size_t SymbolId;

typedef struct Expression Expression;

typedef enum {
    TYPE_UNIT,
    TYPE_INT,
} Type;

typedef struct {
    bool annotated;
    Type type;
    FilePosition pos;
} TypeAnnotation;

typedef struct {
    bool resolved;
    Type type;
} InferredType;

typedef struct {
    StrSlice name;
    SymbolId symbol;
} Variable;

typedef struct {
    enum {
        BINOP_ASSIGN,
        BINOP_ADD,
    } kind;
    Expression *lhs;
    Expression *rhs;
} BinaryOp;

typedef struct {
    Expression **items;
    size_t len;
} Arguments;

typedef struct {
    StrSlice function;
    SymbolId symbol;
    Arguments args;
} FunctionCall;

typedef struct {
    StrSlice name;
    SymbolId symbol;
    TypeAnnotation type_annotation;
    Expression *expr;
    bool mutable;
} VariableDefinition;

typedef struct {
    Expression **items;
    size_t len;
} Block;

struct Expression {
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
        Variable variable;
        BinaryOp binop;
        FunctionCall funcall;
        VariableDefinition vardef;
        Block block;
    };
    FilePosition pos;
    InferredType inferred_type;
};

typedef struct {
    StrSlice name;
    SymbolId symbol;
    TypeAnnotation type_annotation;
    bool mutable;
    FilePosition pos;
} Parameter;

typedef struct {
    Parameter *items;
    size_t len;
} Parameters;

typedef struct {
    StrSlice name;
    SymbolId symbol;
    Parameters params;
    TypeAnnotation return_type_annotation;
    Expression *body;
    FilePosition pos;
} Function;

typedef struct {
    Function *functions;
    size_t function_count;
} Program;

#endif // AST_H_
