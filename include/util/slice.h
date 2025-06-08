#ifndef SLICE_H_
#define SLICE_H_

#include <stddef.h>

#include "util/source.h"

typedef struct {
    const char *ptr;
    const size_t len;
} Slice;

Slice slice(const char *str, size_t start, size_t len);
Slice slice_from_location(const char *buffer, Location loc);
bool slice_eq_str(Slice slice, char *str);
void slice_print(Slice slice);

#endif // SLICE_H_
