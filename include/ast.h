#ifndef AST_H_
#define AST_H_

#include <stdint.h>

#include "util/slice.h"

typedef size_t SymbolId;

typedef enum Type Type;
typedef struct TypeAnnotation TypeAnnotation;
typedef struct BinaryOp BinaryOp;
typedef struct ArgumentList ArgumentList;
typedef struct FunctionCall FunctionCall;
typedef struct VariableDefinition VariableDefinition;
typedef struct Block Block;
typedef struct Expression Expression;
typedef struct Parameter Parameter;
typedef struct ParameterList ParameterList;
typedef struct Function Function;
typedef struct Program Program;

enum Type {
    TYPE_UNIT,
    TYPE_INT,
};

struct TypeAnnotation {
    bool annotated;
    Type type;
};

struct BinaryOp {
    enum {
        BINOP_ASSIGN,
        BINOP_ADD,
    } kind;
    Expression *lhs;
    Expression *rhs;
};

struct ArgumentList {
    Expression *head;
    struct ArgumentList *tail;
};

struct FunctionCall {
    Slice function;
    SymbolId symbol;
    ArgumentList *args;
};

struct VariableDefinition {
    Slice name;
    SymbolId symbol;
    TypeAnnotation type_annotation;
    Expression *expr;
    bool mutable;
};

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
        struct {
            Slice name;
            SymbolId symbol;
        } variable;
        BinaryOp binop;
        FunctionCall funcall;
        VariableDefinition vardef;
        Block *block;
    };
};

struct Parameter {
    Slice name;
    SymbolId symbol;
    Type type;
    bool mutable;
};

struct ParameterList {
    Parameter head;
    struct ParameterList *tail;
};

struct Function {
    Slice name;
    SymbolId symbol;
    ParameterList *params;
    Type return_type;
    Expression *body;
};

struct Program {
    Function head;
    Program *tail;
};

#endif // AST_H_
