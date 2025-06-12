#ifndef AST_H_
#define AST_H_

#include <stdint.h>

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
} TypeAnnotation;

typedef struct {
    Slice name;
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

typedef struct ArgumentList ArgumentList;
struct ArgumentList {
    Expression *head;
    struct ArgumentList *tail;
};

typedef struct {
    Slice function;
    SymbolId symbol;
    ArgumentList *args;
} FunctionCall;

typedef struct {
    Slice name;
    SymbolId symbol;
    TypeAnnotation type_annotation;
    Expression *expr;
    bool mutable;
} VariableDefinition;

typedef struct Block Block;
struct Block {
    Expression *head;
    struct Block *tail;
};

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
};

typedef struct {
    Slice name;
    SymbolId symbol;
    Type type;
    bool mutable;
} Parameter;

typedef struct ParameterList ParameterList;
struct ParameterList {
    Parameter head;
    struct ParameterList *tail;
};

typedef struct {
    Slice name;
    SymbolId symbol;
    ParameterList *params;
    Type return_type;
    Expression *body;
} Function;

typedef struct Program Program;
struct Program {
    Function head;
    Program *tail;
};

#endif // AST_H_
