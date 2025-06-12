#ifndef SOURCE_H_
#define SOURCE_H_

#include <stddef.h>

typedef struct {
    char *filename;
    char *buffer;
    size_t len;
} SourceContext;

typedef struct {
    size_t start, end;
} Location;

typedef struct {
    size_t row, column;
} FilePosition;

SourceContext source_context(char *buffer, char *filename);
FilePosition file_position(size_t index, SourceContext source);

SourceContext read_source_from_file(char *path);

#endif // SOURCE_H_
