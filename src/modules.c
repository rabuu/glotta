#include "modules.h"

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "io.h"
#include "string.h"

ModuleTable module_table_init(size_t count) {
    ModuleTable table = {
        .modules = malloc(count * sizeof(Module)),
        .count = count,
    };
    return table;
}

typedef struct {
    ModuleTable table;
    size_t count;
} ReadModuleContext;

void read_module(const char *path, void *custom_data) {
    ReadModuleContext *ctx = custom_data;

    size_t len;
    char *source = read_file_to_string(path, &len);

    Module new_module = {
        .id = ctx->count,
        .name = "placeholder",
        .full_path = strdup(path),
        .source = source,
        .len = len,
    };

    ctx->table.modules[ctx->count] = new_module;
    ctx->count += 1;
}

void count_modules(const char *, void *custom_data) {
    size_t *counter = custom_data;
    *counter += 1;
}

ModuleTable read_modules(const char *projectdir) {
    size_t counter = 0;
    walk_project_tree(projectdir, count_modules, &counter);

    ReadModuleContext ctx = {
        .table = module_table_init(counter),
        .count = 0,
    };

    walk_project_tree(projectdir, read_module, &ctx);
    return ctx.table;
}

void print_module(Module *module) {
    printf("Module %s, ID %zu, path %s\n----------------------\n%s", module->name, module->id,
           module->full_path, module->source);
}
