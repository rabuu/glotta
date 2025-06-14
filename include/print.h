#ifndef PRINT_H_
#define PRINT_H_

#include "ast.h"
#include "lexing.h"
#include "source.h"
#include "util/slice.h"

void print_strslice(StrSlice slice);
void print_strslice_err(StrSlice slice);

void print_token(Token *token, SourceContext source);

void print_type(Type type);
void print_expression(Expression *expr);
void print_function(Function *fun);
void print_program(Program *program);

#endif // PRINT_H_
