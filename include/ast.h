#ifndef AST_H_
#define AST_H_

#include <stdint.h>

#include "source.h"
#include "strslice.h"

typedef size_t SymbolId;

typedef struct AST_Expr AST_Expr;

typedef enum {
    TYPE_UNIT,
    TYPE_INT,
} AST_Type;

typedef struct {
    bool annotated;
    AST_Type type;
    FilePosition pos;
} AST_TypeAnnotation;

typedef struct {
    bool resolved;
    AST_Type type;
} AST_InferredType;

typedef struct {
    StrSlice name;
    SymbolId symbol;
} AST_Variable;

typedef struct {
    enum {
        BINOP_ASSIGN,
        BINOP_ADD,
    } kind;
    AST_Expr *lhs;
    AST_Expr *rhs;
} AST_BinaryOp;

typedef struct {
    AST_Expr **items;
    size_t len;
} AST_Arguments;

typedef struct {
    StrSlice function;
    SymbolId symbol;
    AST_Arguments args;
} AST_FunCall;

typedef struct {
    StrSlice name;
    SymbolId symbol;
    AST_TypeAnnotation type_annotation;
    AST_Expr *expr;
    bool mutable;
} AST_VarDef;

typedef struct {
    AST_Expr **items;
    size_t len;
} AST_Block;

struct AST_Expr {
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
        AST_Variable variable;
        AST_BinaryOp binop;
        AST_FunCall funcall;
        AST_VarDef vardef;
        AST_Block block;
    };
    FilePosition pos;
    AST_InferredType inferred_type;
};

typedef struct {
    StrSlice name;
    SymbolId symbol;
    AST_TypeAnnotation type_annotation;
    bool mutable;
    FilePosition pos;
} AST_Param;

typedef struct {
    AST_Param *items;
    size_t len;
} AST_Parameters;

typedef struct {
    StrSlice name;
    SymbolId symbol;
    AST_Parameters params;
    AST_TypeAnnotation return_type_annotation;
    AST_Expr *body;
    FilePosition pos;
} AST_Function;

typedef struct {
    AST_Function *functions;
    size_t function_count;
} AST_Program;

#endif // AST_H_
