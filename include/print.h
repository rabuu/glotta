#ifndef PRINT_H_
#define PRINT_H_

#include "ast.h"
#include "lexing.h"
#include "source.h"
#include "util/slice.h"

void print_strslice(StrSlice slice);
void print_strslice_err(StrSlice slice);

void print_token(Token *token, SourceContext source);

void print_type(AstType type);
void print_expression(AstExpr *expr);
void print_function(AstFunction *fun);
void print_program(AstProgram *program);

#endif // PRINT_H_
