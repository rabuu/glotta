/* ARENA
 * -----
 * https://github.com/tsoding/arena/blob/master/arena.h
 */

#include "arena.h"

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

ArenaRegion *arena_region(size_t cap) {
    size_t size_bytes = sizeof(ArenaRegion) + sizeof(uintptr_t) * cap;
    ArenaRegion *region = malloc(size_bytes);
    memset(region, 0, size_bytes);

    region->next = nullptr;
    region->count = 0;
    region->cap = cap;
    return region;
}

void arena_region_free(ArenaRegion *region) { free(region); }

void *arena_alloc(Arena *arena, size_t size_bytes) {
    size_t size = (size_bytes + sizeof(uintptr_t) - 1) / sizeof(uintptr_t);

    if (arena->end == nullptr) {
        size_t cap = ARENA_REGION_DEFAULT_CAPACITY;
        if (cap < size) { cap = size; }

        arena->end = arena_region(cap);
        arena->begin = arena->end;
    }

    while (arena->end->count + size > arena->end->cap && arena->end->next != nullptr) {
        arena->end = arena->end->next;
    }

    if (arena->end->count + size > arena->end->cap) {
        size_t cap = ARENA_REGION_DEFAULT_CAPACITY;
        if (cap < size) { cap = size; }

        arena->end->next = arena_region(cap);
        arena->end = arena->end->next;
    }

    void *result = &arena->end->data[arena->end->count];
    arena->end->count += size;
    return result;
}

void arena_free(Arena *arena) {
    ArenaRegion *region = arena->begin;
    while (region) {
        ArenaRegion *r = region;
        region = region->next;
        arena_region_free(r);
    }

    arena->begin = nullptr;
    arena->end = nullptr;
}
