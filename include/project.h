#ifndef PROJECT_H_
#define PROJECT_H_

#include <stddef.h>

#include "strslice.h"

typedef struct {
    size_t id;
    char *fs_path;

    StrSlice name;
    StrSlice *module_path;
    size_t module_path_len;

    char *src;
    size_t src_len;
} Module;

typedef struct {
    Module *modules;
    size_t module_count;
} Project;

Project read_project(const char *path);
Project read_multi_file_project(const char *projectdir);
Project read_single_file_project(const char *filepath);

void project_free(Project *project);

void print_module(Module *module);

#endif // !PROJECT_H_
