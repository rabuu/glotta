#ifndef PRINT_H_
#define PRINT_H_

#include "ast.h"
#include "lexer.h"
#include "source.h"
#include "util/slice.h"

void print_slice(Slice slice);
void print_slice_err(Slice slice);

void print_token(Token *token, SourceContext source);

void print_expression(Expression *expr);
void print_function(Function *fun);
void print_program(Program *program);

#endif // PRINT_H_
