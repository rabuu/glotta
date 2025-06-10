#ifndef PARSER_H_
#define PARSER_H_

#include "ast.h"
#include "lexer.h"
#include "util/arena.h"

Function parse_function(Lexer *lexer, Arena *a);
Expression *parse_expr(Lexer *lexer, Arena *a);

#endif // PARSER_H_
