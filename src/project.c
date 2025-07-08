#include "project.h"

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "arena.h"
#include "io.h"
#include "string.h"

Project project_init(size_t count) {
    Project table = {
        .modules = malloc(count * sizeof(Module)),
        .module_count = count,
    };
    return table;
}

void project_free(Project *project) {
    for (size_t i = 0; i < project->module_count; ++i) {
        free(project->modules[i].fs_path);
    }

    free(project->modules);
}

bool is_glotta_path(const char *path) {
    const char *suffix = ".glotta";
    size_t len = strlen(path);
    size_t suffix_len = strlen(suffix);
    if (len < suffix_len) { return false; }
    return strcmp(path + len - suffix_len, suffix) == 0;
}

typedef ARENA_ARRAY(char *) Paths;

struct FindModulesContext {
    Paths paths;
};

void find_modules(const char *path, void *custom_data) {
    if (!is_glotta_path(path)) { return; }

    struct FindModulesContext *ctx = custom_data;
    arena_array_push(ctx->paths, strdup(path));
}

Project read_multi_file_project(const char *projectdir) {
    Arena paths_arena = {0};
    Paths paths;
    arena_array_init(paths, &paths_arena);

    struct FindModulesContext ctx = {.paths = paths};
    walk_project_tree(projectdir, find_modules, &ctx);

    size_t name_offset = strlen(projectdir);

    Project project = project_init(ctx.paths.len);
    for (size_t i = 0; i < ctx.paths.len; ++i) {
        char *module_path = ctx.paths.data[i];

        size_t len;
        char *source = read_file_to_string(module_path, &len);

        Module module = {
            .id = 0,
            .fs_path = module_path,
            .name = module_path + name_offset + 1,
            .source = source,
            .len = len,
        };

        project.modules[i] = module;
    }

    arena_free(&paths_arena);

    return project;
}

Project read_single_file_project(const char *filepath) {
    size_t len;
    char *source = read_file_to_string(filepath, &len);

    Module module = {
        .id = 0,
        .fs_path = (char *)filepath,
        .name = "main",
        .source = source,
        .len = len,
    };

    Project project = project_init(1);
    project.modules[0] = module;

    return project;
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
           module->fs_path, module->source);
}
