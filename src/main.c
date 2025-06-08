#include <stdio.h>
#include <stdlib.h>

#include "lexer.h"
#include "parser.h"
#include "util/source.h"

char *read_file_to_string(const char *filename) {
    FILE *f = fopen(filename, "rb");
    if (!f) {
        perror("Failed to open file");
        return NULL;
    }

    if (fseek(f, 0, SEEK_END) != 0) {
        perror("fseek failed");
        fclose(f);
        return NULL;
    }

    long filesize = ftell(f);
    if (filesize < 0) {
        perror("ftell failed");
        fclose(f);
        return NULL;
    }
    rewind(f);

    char *buffer = malloc(filesize + 1);
    if (!buffer) {
        perror("malloc failed");
        fclose(f);
        return NULL;
    }

    long read_size = fread(buffer, 1, filesize, f);
    if (read_size != filesize) {
        perror("fread failed");
        free(buffer);
        fclose(f);
        return NULL;
    }
    buffer[filesize] = '\0';

    fclose(f);
    return buffer;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "Usage: glotta SOURCE-FILE.glotta\n");
        exit(1);
    }

    char *path = argv[1];
    printf("PATH: %s\n------------------------\n", path);

    char *source = read_file_to_string(path);
    printf("%s\n------------------------\n", source);

    SourceContext source_ctx = source_context(source, path);
    Lexer lexer = lexer_init(source_ctx);

    Token tok;
    do {
        tok = lexer_next(&lexer);
        token_debug(&tok, source_ctx);
    } while (tok.tag != TOK_EOF);
    printf("------------------------\n");

    lexer.index = 0;
    parse_function(&lexer);

    return 0;
}
