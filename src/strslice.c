#include "strslice.h"

#include <string.h>

StrSlice strslice(char *str, size_t start, size_t len) {
    return (StrSlice){
        .ptr = str + start,
        .len = len,
    };
}

StrSlice strslice_from_str(char *str) {
    return (StrSlice){
        .ptr = str,
        .len = strlen(str),
    };
}

StrSlice strslice_from_str_off(char *str, size_t offset) {
    return (StrSlice){
        .ptr = str + offset,
        .len = strlen(str) - offset,
    };
}

StrSlice strslice_from_loc(char *buffer, Location loc) {
    size_t len = loc.end - loc.start;
    return strslice(buffer, loc.start, len);
}

bool strslice_eq(StrSlice a, StrSlice b) {
    if (a.len != b.len) { return false; }
    return memcmp(a.ptr, b.ptr, a.len) == 0;
}

bool strslice_eq_str(StrSlice slice, const char *str) {
    if (slice.len != strlen(str)) { return false; }
    return memcmp(slice.ptr, str, slice.len) == 0;
}

size_t strslice_count(StrSlice slice, char c) {
    size_t counter = 0;

    for (size_t i = 0; i < slice.len; ++i) {
        if (slice.ptr[i] == c) { counter++; }
    }

    return counter;
}

size_t strslice_find(StrSlice slice, char c) {
    for (size_t i = 0; i < slice.len; ++i) {
        if (slice.ptr[i] == c) { return i; }
    }

    return -1;
}

bool strslice_has_suffix(StrSlice slice, const char *suffix) {
    size_t suffix_len = strlen(suffix);
    StrSlice slice_suffix = strslice(slice.ptr, slice.len - suffix_len, suffix_len);
    return strslice_eq_str(slice_suffix, suffix);
}

bool strslice_trim_suffix(StrSlice *slice, const char *suffix) {
    if (!strslice_has_suffix(*slice, suffix)) { return false; }

    size_t suffix_len = strlen(suffix);
    slice->len = slice->len - suffix_len;

    return true;
}

void strslice_bump(StrSlice *slice, size_t offset) {
    slice->ptr += offset;
    slice->len -= offset;
}

bool strslice_forall(StrSlice slice, bool (*p)(char c)) {
    for (size_t i = 0; i < slice.len; ++i) {
        bool pred = p(slice.ptr[i]);
        if (!pred) { return false; }
    }

    return true;
}
