#ifndef AST_H_
#define AST_H_

#include <stdint.h>

#include "util.h"

typedef enum Type Type;
typedef struct TypeAnnotation TypeAnnotation;
typedef struct BinaryOp BinaryOp;
typedef struct Assignment Assignment;
typedef struct Block Block;
typedef struct Expression Expression;
typedef struct Parameter Parameter;
typedef struct ParameterList ParameterList;
typedef struct Function Function;

enum Type {
    UNIT,
    INT,
};

struct TypeAnnotation {
    bool annotated;
    Type type;
};

struct BinaryOp {
    enum {
        BINOP_ADD,
    } kind;
    Expression *lhs;
    Expression *rhs;
};

struct Assignment {
    Slice name;
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
        EXPR_INTEGER,
        EXPR_VARIABLE,
        EXPR_BINOP,
        EXPR_FUNCALL,
        EXPR_ASSIGNMENT,
        EXPR_BLOCK,
    } tag;
    union {
        int32_t integer;
        Slice variable;
        BinaryOp binop;
        Assignment assignment;
        Block block;
    };
};

struct Parameter {
    Slice name;
    Type type;
    bool mutable;
};

struct ParameterList {
    Parameter head;
    struct ParameterList *tail;
};

struct Function {
    Slice name;
    ParameterList params;
    Type return_type;
    Expression *body;
};

#endif // AST_H_
