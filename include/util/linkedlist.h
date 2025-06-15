#ifndef LINKEDLIST_H_
#define LINKEDLIST_H_

#define GENERATE_LINKEDLIST_TYPE(T, Name, name)                                                    \
    typedef struct Name {                                                                          \
        T head;                                                                                    \
        struct Name *tail;                                                                         \
    } Name;                                                                                        \
    static inline size_t name##_count(Name *list) {                                                \
        if (!list) { return 0; }                                                                   \
        return 1 + name##_count(list->tail);                                                       \
    }

#endif // LINKEDLIST_H_
