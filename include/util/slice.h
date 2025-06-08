#ifndef SLICE_H_
#define SLICE_H_

#include <stddef.h>

#include "util/location.h"

typedef struct {
    char *ptr;
    size_t len;
} Slice;

Slice slice(char *str, size_t start, size_t len);
Slice slice_from_source(char *source, Location loc);
bool slice_eq_str(Slice slice, char *str);
void slice_print(Slice slice);

#endif // SLICE_H_
