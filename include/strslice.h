#ifndef STRSLICE_H_
#define STRSLICE_H_

#include <stddef.h>

#include "source.h"

typedef struct {
    char *ptr;
    size_t len;
} StrSlice;

StrSlice strslice(char *str, size_t start, size_t len);
StrSlice strslice_from_loc(char *buffer, Location loc);
bool strslice_eq(StrSlice a, StrSlice b);
bool strslice_eq_str(StrSlice slice, char *str);

#endif // STRSLICE_H_
