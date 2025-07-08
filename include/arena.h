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
#define ARENA_REGION_DEFAULT_CAPACITY (8 * 1024)
#endif // ARENA_REGION_DEFAULT_CAPACITY

ArenaRegion *arena_region(size_t cap);
void arena_region_free(ArenaRegion *region);

void *arena_alloc(Arena *arena, size_t size_bytes);
void arena_free(Arena *arena);

#define ARENA_ARRAY(T)                                                                             \
    struct {                                                                                       \
        T *data;                                                                                   \
        size_t len, cap;                                                                           \
        Arena *arena;                                                                              \
    }

#define arena_array_init(arr, a)                                                                   \
    do {                                                                                           \
        (arr).data = nullptr;                                                                      \
        (arr).len = 0;                                                                             \
        (arr).cap = 0;                                                                             \
        (arr).arena = (a);                                                                         \
    } while (0)

#define arena_array_push(arr, val)                                                                 \
    do {                                                                                           \
        if ((arr).len == (arr).cap) {                                                              \
            size_t newcap = (arr).cap ? (arr).cap * 2 : 8;                                         \
            void *newdata = arena_alloc((arr).arena, sizeof(*(arr).data) * newcap);                \
            if ((arr).data) memcpy(newdata, (arr).data, sizeof(*(arr).data) * (arr).len);          \
            (arr).data = newdata;                                                                  \
            (arr).cap = newcap;                                                                    \
        }                                                                                          \
        (arr).data[(arr).len++] = (val);                                                           \
    } while (0)

#endif // !ARENA_H_
