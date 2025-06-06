#include "util.h"
#include <stdio.h>
#include <string.h>

Slice slice(char *str, size_t start, size_t len) {
    return (Slice){
        .ptr = str + start,
        .len = len,
    };
}


Slice slice_from_source(char *source, SourcePosition pos) {
    size_t len = pos.end - pos.start;
    return slice(source, pos.start, len);
}

bool slice_eq_str(Slice slice, char *str) {
    if (slice.len != strlen(str)) { return false; }
    return memcmp(slice.ptr, str, slice.len) == 0;
}

void slice_print(Slice slice) {
    for (size_t i = 0; i < slice.len; ++i) {
        char c = *(slice.ptr + i);
        putchar(c);
    }
}
