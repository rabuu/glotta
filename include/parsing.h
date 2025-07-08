#ifndef PARSER_H_
#define PARSER_H_

#include "arena.h"
#include "ast.h"
#include "lexing.h"

AST_Program parse_program(Lexer *lexer, Arena *ast_arena);

#endif // PARSER_H_
