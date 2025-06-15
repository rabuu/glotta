#include "util/slice.h"

#include <string.h>

StrSlice strslice(char *str, size_t start, size_t len) {
    return (StrSlice){
        .ptr = str + start,
        .len = len,
    };
}

StrSlice strslice_from_loc(char *buffer, Location loc) {
    size_t len = loc.end - loc.start;
    return strslice(buffer, loc.start, len);
}

bool strslice_eq_str(StrSlice slice, char *str) {
    if (slice.len != strlen(str)) { return false; }
    return memcmp(slice.ptr, str, slice.len) == 0;
}
