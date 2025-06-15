#ifndef PARSER_H_
#define PARSER_H_

#include "ast.h"
#include "lexer.h"
#include "util/arena.h"

Program parse_program(Lexer *lexer, Arena *ast_arena);

#endif // PARSER_H_
