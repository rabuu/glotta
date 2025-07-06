#ifndef PRINT_H_
#define PRINT_H_

#include "ast.h"
#include "lexing.h"
#include "source.h"
#include "util/slice.h"

void print_strslice(StrSlice slice);
void print_strslice_err(StrSlice slice);

void print_token(Token *token, SourceContext source);

void print_type(AST_Type type);
void print_expression(AST_Expr *expr);
void print_function(AST_Function *fun);
void print_program(AST_Program *program);

#endif // PRINT_H_
