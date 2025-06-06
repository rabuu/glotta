#ifndef UTIL_H_
#define UTIL_H_

#include <stddef.h>

typedef struct {
    size_t start, end;
} SourcePosition;

typedef struct {
    char *ptr;
    size_t len;
} Slice;

Slice slice(char *str, size_t start, size_t len);
bool slice_eq_str(Slice slice, char *str);

#endif // UTIL_H_
