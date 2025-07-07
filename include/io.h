#ifndef IO_H_
#define IO_H_

#include <stddef.h>
typedef void (*FileCallback)(const char *path, void *custom_data);

bool is_dir(const char *path);
bool is_glotta_path(const char *path);

bool walk_project_tree(const char *dirpath, FileCallback on_file, void *custom_data);

char *read_file_to_string(const char *path, size_t *len);

#endif // IO_H_
