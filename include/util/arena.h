/* ARENA
 * -----
 * https://github.com/tsoding/arena/blob/master/arena.h
 */

#ifndef ARENA_H_
#define ARENA_H_

#include <stddef.h>
#include <stdint.h>

typedef struct ArenaRegion ArenaRegion;

struct ArenaRegion {
	ArenaRegion *next;
	size_t count;
	size_t cap;
	uintptr_t data[];
};

typedef struct {
	ArenaRegion *begin, *end;
} Arena;

#ifndef ARENA_REGION_DEFAULT_CAPACITY
#define ARENA_REGION_DEFAULT_CAPACITY (8*1024)
#endif // ARENA_REGION_DEFAULT_CAPACITY

ArenaRegion *arena_region(size_t cap);
void arena_region_free(ArenaRegion *region);

void *arena_alloc(Arena *arena, size_t size_bytes);
void arena_free(Arena *arena);

#endif // !ARENA_H_
