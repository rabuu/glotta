#ifndef AST_H_
#define AST_H_

#include <stdint.h>

#include "source.h"
#include "util/linkedlist.h"
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

GENERATE_LINKEDLIST_TYPE(Expression *, ArgumentList, args)

typedef struct {
    StrSlice function;
    SymbolId symbol;
    ArgumentList *args;
} FunctionCall;

typedef struct {
    StrSlice name;
    SymbolId symbol;
    TypeAnnotation type_annotation;
    Expression *expr;
    bool mutable;
} VariableDefinition;

GENERATE_LINKEDLIST_TYPE(Expression *, Block, block)

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
        Block *block;
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

GENERATE_LINKEDLIST_TYPE(Parameter, ParameterList, params)

typedef struct {
    StrSlice name;
    SymbolId symbol;
    ParameterList *params;
    TypeAnnotation return_type_annotation;
    Expression *body;
    FilePosition pos;
} Function;

GENERATE_LINKEDLIST_TYPE(Function, Program, program)

#endif // AST_H_
