#ifndef Vec_H
#define Vec_H

#include "dt.h"

typedef struct Vec {
    usize item_size;
    void* buffer;
    usize item_count;
    usize item_capacity;
} Vec;

Vec Vec__create(usize item_size);

void Vec__push(Vec* restrict vec, void* restrict item);

u8 Vec__pop(Vec* restrict vec, void* restrict dest_item);

static inline void* Vec__buffer(Vec* restrict vec) {
    return vec->buffer;
}

static inline usize Vec__length(Vec* restrict vec) {
    return vec->item_count;
}

void Vec__grow(Vec* restrict vec, usize min_capacity);

void Vec__clear(Vec* restrict vec);

static inline void* Vec__index(Vec* restrict vec, usize index) {
    return vec->buffer + vec->item_size * index;
}

void Vec__push_all(Vec* restrict vec, void* restrict items, usize item_count);

u8 Vec__swap_remove(Vec* restrict vec, usize index, void* restrict dest_item);

#endif