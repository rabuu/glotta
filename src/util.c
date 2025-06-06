#include "util.h"
#include <string.h>

Slice slice(char *str, size_t start, size_t len) {
    return (Slice){
        .ptr = str + start,
        .len = len,
    };
}

bool slice_eq_str(Slice slice, char *str) {
    if (slice.len != strlen(str)) { return false; }
    return memcmp(slice.ptr, str, slice.len) == 0;
}
