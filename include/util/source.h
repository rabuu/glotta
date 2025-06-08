#ifndef SOURCE_H_
#define SOURCE_H_

#include <stddef.h>

typedef struct {
    const char *filename;
    const char *buffer;
    const size_t len;
} SourceContext;

typedef struct {
    size_t start, end;
} Location;

typedef struct {
    size_t row, column;
} FileLocation;

SourceContext source_context(char *buffer, char *filename);
FileLocation file_location(size_t index, SourceContext source);

#endif // SOURCE_H_
