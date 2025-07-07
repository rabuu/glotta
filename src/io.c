#include "io.h"

#include <dirent.h>
#include <errno.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

bool is_dir(const char *path) {
    struct stat st;
    if (stat(path, &st) != 0) {
        fprintf(stderr, "stat failed on '%s': %s\n", path, strerror(errno));
        return false;
    }

    return S_ISDIR(st.st_mode);
}

bool is_glotta_path(const char *path) {
    const char *suffix = ".glotta";
    size_t len = strlen(path);
    size_t suffix_len = strlen(suffix);
    if (len < suffix_len) { return false; }
    return strcmp(path + len - suffix_len, suffix) == 0;
}

bool walk_project_tree(const char *dirpath, FileCallback on_file, void *custom_data) {
    DIR *dir = opendir(dirpath);
    if (!dir) {
        fprintf(stderr, "Failed to open directory '%s': %s\n", dirpath, strerror(errno));
        return false;
    }

    struct dirent *entry;
    while ((entry = readdir(dir))) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) { continue; }

        char path[4096];
        snprintf(path, sizeof(path), "%s/%s", dirpath, entry->d_name);

        struct stat st;
        if (stat(path, &st) != 0) {
            fprintf(stderr, "stat failed on '%s': %s\n", path, strerror(errno));
            continue;
        }

        if (S_ISDIR(st.st_mode)) {
            walk_project_tree(path, on_file, custom_data);
        } else if (S_ISREG(st.st_mode) && is_glotta_path(path)) {
            on_file(path, custom_data);
        }
    }

    closedir(dir);
    return true;
}

char *read_file_to_string(const char *path, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) { return NULL; }

    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return NULL;
    }

    long filesize = ftell(f);
    if (filesize < 0) {
        fclose(f);
        return NULL;
    }
    rewind(f);

    char *buffer = malloc(filesize + 1);
    if (!buffer) {
        fclose(f);
        return NULL;
    }

    long read_size = fread(buffer, 1, filesize, f);
    if (read_size != filesize) {
        free(buffer);
        fclose(f);
        return NULL;
    }
    buffer[filesize] = '\0';

    fclose(f);

    if (len) { *len = filesize; }
    return buffer;
}
