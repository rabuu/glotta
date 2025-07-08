#include "project.h"

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "io.h"
#include "string.h"

Project project_init(size_t count) {
    Project table = {
        .modules = malloc(count * sizeof(Module)),
        .module_count = count,
    };
    return table;
}

struct ProjectCreationContext {
    Project project;
    size_t count;
};

void read_module(const char *path, void *custom_data) {
    struct ProjectCreationContext *ctx = custom_data;

    size_t len;
    char *source = read_file_to_string(path, &len);

    Module new_module = {
        .id = ctx->count,
        .name = "placeholder",
        .full_path = strdup(path),
        .source = source,
        .len = len,
    };

    ctx->project.modules[ctx->count] = new_module;
    ctx->count += 1;
}

void count_modules(const char *, void *custom_data) {
    size_t *counter = custom_data;
    *counter += 1;
}

Project read_multi_file_project(const char *projectdir) {
    size_t counter = 0;
    walk_project_tree(projectdir, count_modules, &counter);

    struct ProjectCreationContext ctx = {
        .project = project_init(counter),
        .count = 0,
    };

    walk_project_tree(projectdir, read_module, &ctx);
    return ctx.project;
}

Project read_single_file_project(const char *filepath) {
    struct ProjectCreationContext ctx = {
        .project = project_init(1),
        .count = 0,
    };

    read_module(filepath, &ctx);
    return ctx.project;
}

Project read_project(const char *path) {
    if (is_dir(path)) {
        return read_multi_file_project(path);
    } else {
        return read_single_file_project(path);
    }
}

void print_module(Module *module) {
    printf("Module %s, ID %zu, path %s\n----------------------\n%s", module->name, module->id,
           module->full_path, module->source);
}
