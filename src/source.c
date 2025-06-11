#include "source.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

SourceContext source_context(char *buffer, char *filename) {
    size_t len = strlen(buffer);
    return (SourceContext){
        .filename = filename,
        .buffer = buffer,
        .len = len,
    };
}

FileLocation file_location(size_t index, SourceContext source) {
    size_t row = 1;
    size_t column = 1;

    for (size_t i = 0; i < source.len; ++i) {
        if (i == index) { break; }

        char c = source.buffer[i];
        if (c == '\n') {
            row++;
            column = 1;
        } else {
            column += 1;
        }
    }

    return (FileLocation){
        .row = row,
        .column = column,
    };
}

char *read_file_to_string(const char *path) {
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
    return buffer;
}

SourceContext read_source_from_file(char *path) {
    char *buffer = read_file_to_string(path);
    if (!buffer) {
        fprintf(stderr, "ERROR: Failed to read file");
        exit(1);
    }

    return source_context(buffer, path);
}
