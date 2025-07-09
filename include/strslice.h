#ifndef STRSLICE_H_
#define STRSLICE_H_

#include <stddef.h>

#include "source.h"

typedef struct {
    char *ptr;
    size_t len;
} StrSlice;

StrSlice strslice(char *str, size_t start, size_t len);
StrSlice strslice_from_str(char *str);
StrSlice strslice_from_str_off(char *str, size_t offset);
StrSlice strslice_from_loc(char *buffer, Location loc);
bool strslice_eq(StrSlice a, StrSlice b);
bool strslice_eq_str(StrSlice slice, const char *str);
size_t strslice_count(StrSlice slice, char c);
size_t strslice_find(StrSlice slice, char c);
bool strslice_has_suffix(StrSlice slice, const char *suffix);
bool strslice_trim_suffix(StrSlice *slice, const char *suffix);
void strslice_bump(StrSlice *slice, size_t offset);
bool strslice_forall(StrSlice slice, bool (*p)(char c));

#endif // STRSLICE_H_
