#include "source.h"
#include "io.h"

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

FilePosition file_position(size_t index, SourceContext source) {
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

    return (FilePosition){
        .row = row,
        .column = column,
    };
}

SourceContext read_source_from_file(char *path) {
    char *buffer = read_file_to_string(path, nullptr);
    if (!buffer) {
        fprintf(stderr, "ERROR: Failed to read file");
        exit(1);
    }

    return source_context(buffer, path);
}
