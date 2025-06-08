#include "util/source.h"

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
