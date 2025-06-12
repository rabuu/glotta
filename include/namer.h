#ifndef NAMER_H_
#define NAMER_H_

#include <stddef.h>

#include "ast.h"
#include "util/arena.h"
#include "util/slice.h"

#define DEFAULT_SYMBOL_TABLE_CAP 4

typedef struct Scope Scope;

typedef enum {
    SYMBOL_FUNCTION,
    SYMBOL_VARIABLE,
} SymbolKind;

typedef struct {
    SymbolKind kind;
    SymbolId id;
    Slice name;
} Symbol;

struct Scope {
    struct Scope *parent;

    Symbol *symbols;

    size_t count;
    size_t cap;
};

Scope *scope_fork(Scope *parent, Arena *a);
void scope_add_symbol(Scope *scope, Symbol symbol, Arena *a);
SymbolId scope_lookup(Scope *scope, Slice name, SymbolKind kind);

void resolve_names(Program *program);

#endif // NAMER_H_
