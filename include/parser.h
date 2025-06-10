#ifndef PARSER_H_
#define PARSER_H_

#include "lexer.h"
#include "util/arena.h"

void parse_function(Lexer *lexer, Arena *a);

#endif // PARSER_H_
