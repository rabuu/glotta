#ifndef SLICE_H_
#define SLICE_H_

#include <stddef.h>

#include "util/source.h"

typedef struct {
    char *ptr;
    size_t len;
} Slice;

Slice slice(char *str, size_t start, size_t len);
Slice slice_from_location(char *buffer, Location loc);
bool slice_eq_str(Slice slice, char *str);

#endif // SLICE_H_
