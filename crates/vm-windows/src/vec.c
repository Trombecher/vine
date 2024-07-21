#include <malloc.h>
#include <memory.h>

#include "vec.h"
#include "dt.h"

usize next_power_of_2(usize const target, usize start) {
    while(start < target)
        start <<= 1;
    return start;
}

Vec Vec__create(usize const item_size) {
    if(item_size == 0) {
        // TODO
        exit(-20);
    }

    const Vec vec = {
        .item_size = item_size,
        .buffer = NULL,
        .item_count = 0,
        .item_capacity = 0,
    };
    return vec;
}

void Vec__push(Vec* restrict const vec, void* restrict const item) {
    // Ensures enough space for one more item.
    Vec__grow(vec, vec->item_count + 1);

    memcpy(Vec__index(vec, vec->item_count), item, vec->item_size);

    ++vec->item_count;
}

void Vec__push_all(Vec* restrict const vec, void* restrict const items, usize const item_count) {
    // Ensures enough space for the items.
    Vec__grow(vec, vec->item_count + item_count);

    memcpy(Vec__index(vec, vec->item_count), items, vec->item_size * item_count);
    vec->item_count += item_count;
}

void Vec__grow(Vec* restrict const vec, usize const min_capacity) {
    if(vec->buffer == NULL) {
        const auto new_capacity = next_power_of_2(min_capacity, 1);
        vec->item_capacity = new_capacity;
        vec->buffer = malloc(vec->item_size * new_capacity);
    } else if(vec->item_capacity < min_capacity) {
        const auto new_capacity = next_power_of_2(min_capacity, vec->item_capacity);
        if(new_capacity == vec->item_capacity) return;

        vec->item_capacity = new_capacity;
        const auto new_buffer = malloc(new_capacity * vec->item_size);

        memcpy(new_buffer, vec->buffer, vec->item_count * vec->item_size);
        free(vec->buffer);
        vec->buffer = new_buffer;
    }
}

u8 Vec__swap_remove(Vec* restrict const vec, usize const index, void* restrict const dest_item) {
    if(index >= Vec__length(vec)) return 0;

    memcpy(
        dest_item,
        Vec__index(vec, index),
        vec->item_size
    );

    vec->item_count--;

    // In the case that `index` is not indexing the last element, we move the last element to the free slot.
    if(index != Vec__length(vec) - 1) {
        memcpy(
            Vec__index(vec, index),
            Vec__index(vec, vec->item_count),
            vec->item_size
        );
    }

    return 1;
}

u8 Vec__pop(Vec* restrict const vec, void* restrict const dest_item) {
    if(vec->item_count == 0) {
        return 0;
    }

    vec->item_count--;
    memcpy(
        dest_item,
        Vec__index(vec, vec->item_count),
        vec->item_size
    );

    return 1;
}

/**
 * Frees the buffer.
 */
void Vec__clear(Vec* restrict const vec) {
    if(vec->buffer != NULL) {
        free(vec->buffer);
        vec->buffer = NULL;
    }
}