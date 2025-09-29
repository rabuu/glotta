#include "project.h"

#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "arena.h"
#include "io.h"
#include "lexing.h"
#include "parsing.h"
#include "print.h"
#include "string.h"
#include "strslice.h"

const char *GLOTTA_FILE_EXT = ".glotta";

Project project_init(size_t count) {
    Project table = {
        .modules = malloc(count * sizeof(Module)),
        .module_count = count,
    };
    return table;
}

void project_free(Project *project) {
    for (size_t i = 0; i < project->module_count; ++i) {
        free(project->modules[i].module_path);
        free(project->modules[i].fs_path);
    }

    free(project->modules);
}

bool is_glotta_path(const char *path) {
    size_t len = strlen(path);
    size_t suffix_len = strlen(GLOTTA_FILE_EXT);
    if (len < suffix_len) { return false; }
    return strcmp(path + len - suffix_len, GLOTTA_FILE_EXT) == 0;
}

bool is_valid_path_component_char(char c) { return (c >= 'a' && c <= 'z') || (c == '_'); }

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
        char *fs_path = ctx.paths.data[i];
        StrSlice project_relative_path = strslice_from_str_off(fs_path, name_offset + 1);

        size_t module_path_len = strslice_count(project_relative_path, '/');
        StrSlice *module_path = malloc(module_path_len * sizeof(StrSlice));

        StrSlice current_path = project_relative_path;
        for (size_t m = 0; m < module_path_len; ++m) {
            size_t next_slash = strslice_find(current_path, '/');
            if (next_slash >= current_path.len) { assert(false); }

            StrSlice path_component = {
                .ptr = current_path.ptr,
                .len = next_slash,
            };

            if (!strslice_forall(path_component, is_valid_path_component_char)) {
                assert(false && "Invalid module path component");
            }

            module_path[m] = path_component;

            strslice_bump(&current_path, next_slash + 1);
        }

        if (!strslice_trim_suffix(&current_path, GLOTTA_FILE_EXT)) { assert(false); }

        if (!strslice_forall(current_path, is_valid_path_component_char)) {
            assert(false && "Invalid module name");
        }

        size_t len;
        char *source = read_file_to_string(fs_path, &len);

        Module module = {
            .id = i,
            .fs_path = fs_path,
            .name = current_path,
            .module_path = module_path,
            .module_path_len = module_path_len,
            .src = source,
            .src_len = len,
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
        .name = strslice_from_str("main"),
        .module_path = nullptr,
        .module_path_len = 0,
        .src = source,
        .src_len = len,
    };

    Project project = project_init(1);
    project.modules[0] = module;

    return project;
}

Project new_project(const char *path) {
    if (is_dir(path)) {
        return read_multi_file_project(path);
    } else {
        return read_single_file_project(path);
    }
}

void parse_module(Module *module) {
    SourceContext source = {
        .filename = "TODO",
        .buffer = module->src,
        .len = module->src_len,
    };

    module->lexer = lexer_init(source);
    module->ast = parse_program(&module->lexer, &module->ast_arena);
}

void parse_project(Project *project) {
    for (size_t i = 0; i < project->module_count; ++i) {
        parse_module(&project->modules[i]);
    }
}

void print_module(Module *module) {
    printf("------------------\nMODULE#%zu ", module->id);
    for (size_t i = 0; i < module->module_path_len; ++i) {
        print_strslice(module->module_path[i]);
        printf(".");
    }
    print_strslice(module->name);
    printf("\n------------------\n");
    print_program(&module->ast);
}

void print_project(Project *project) {
    for (size_t i = 0; i < project->module_count; ++i) {
        print_module(&project->modules[i]);
    }
}
