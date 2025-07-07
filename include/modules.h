#ifndef MODULES_H_
#define MODULES_H_

#include <stddef.h>

typedef struct {
    size_t id;
    const char *name;
    const char *full_path;

    char *source;
    size_t len;
} Module;

typedef struct {
    Module *modules;
    const size_t count;
} ModuleTable;

ModuleTable read_modules(const char *projectdir);

void print_module(Module *module);

#endif // !MODULES_H_
