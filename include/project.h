#ifndef PROJECT_H_
#define PROJECT_H_

#include <stddef.h>

#include "arena.h"
#include "ast.h"
#include "lexing.h"
#include "strslice.h"

typedef struct {
    size_t id;
    char *fs_path;

    StrSlice name;
    StrSlice *module_path;
    size_t module_path_len;

    char *src;
    size_t src_len;

    Lexer lexer;
    Arena ast_arena;
    AST_Program ast;
} Module;

typedef struct {
    Module *modules;
    size_t module_count;
} Project;

Project new_project(const char *path);
void project_free(Project *project);

void parse_project(Project *project);

void print_project(Project *project);

#endif // !PROJECT_H_
