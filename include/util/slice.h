#ifndef SLICE_H_
#define SLICE_H_

#include <stddef.h>

#include "source.h"

#define GENERATE_SLICE_TYPE(T)                                                                     \
    struct {                                                                                       \
        T *ptr;                                                                                    \
        size_t len;                                                                                \
    }

#define slice_eq(a, b) (((a).len != (b).len) ? false : memcmp((a).ptr, (b).ptr, (a).len) == 0)

typedef GENERATE_SLICE_TYPE(char) StrSlice;

StrSlice strslice(char *str, size_t start, size_t len);
StrSlice strslice_from_loc(char *buffer, Location loc);
bool strslice_eq_str(StrSlice slice, char *str);

#endif // SLICE_H_
