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
Slice slice_from_source(char *source, SourcePosition pos);
bool slice_eq_str(Slice slice, char *str);
void slice_print(Slice slice);

#endif // UTIL_H_
