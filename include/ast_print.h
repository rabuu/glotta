#ifndef AST_PRINT_H_
#define AST_PRINT_H_

#include "ast.h"

void print_expression(Expression *expr);
void print_function(Function *fun);
void print_program(Program *program);

#endif // AST_PRINT_H_
