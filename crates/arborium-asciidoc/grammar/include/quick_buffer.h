#pragma once
#include <string.h>
#include "../tree_sitter/parser.h"
#include "base_types.h"

#ifndef bzero
#define bzero(dst, len) memset(dst, 0, len)
#endif  // !bzero

typedef enum Result {
    RESULT_ERR = 0,
    RESULT_OK = 1,
} Result;

typedef struct QuickBuffer {
    void* buffer;
    usize pos;
    usize capacity;
} QuickBuffer;

static QuickBuffer quick_buffer_new(void* buffer, usize capacity) {
    QuickBuffer b;
    b.buffer = buffer;
    b.pos = 0;
    b.capacity = capacity;

    return b;
}

#define impl_write_for(ty)                                                      \
    static inline Result quick_buffer_write_##ty(QuickBuffer* self, ty value) { \
        if(self->capacity - self->pos < sizeof(ty)) {                           \
            return RESULT_ERR;                                                  \
        }                                                                       \
                                                                                \
        memcpy((u8*)self->buffer + self->pos, &value, sizeof(ty));              \
        self->pos += sizeof(ty);                                                \
                                                                                \
        return RESULT_OK;                                                       \
    }

impl_write_for(bool);
impl_write_for(u8);
impl_write_for(i8);
impl_write_for(u32);
impl_write_for(usize);

#define impl_read_for(ty)                                                       \
    static inline Result quick_buffer_read_##ty(QuickBuffer* self, ty* value) { \
        if(self->capacity - self->pos < sizeof(ty)) {                           \
            return RESULT_ERR;                                                  \
        }                                                                       \
                                                                                \
        memcpy(value, (u8*)self->buffer + self->pos, sizeof(ty));               \
        self->pos += sizeof(ty);                                                \
                                                                                \
        return RESULT_OK;                                                       \
    }

impl_read_for(bool);
impl_read_for(u8);
impl_read_for(i8);
impl_read_for(u32);
impl_read_for(usize);

static inline Result quick_buffer_extend_bytes(QuickBuffer* self, void const* buffer, usize len) {
    if(self->capacity - self->pos < len) {
        return RESULT_ERR;
    }

    memcpy((u8*)self->buffer + self->pos, buffer, len);
    self->pos += len;

    return RESULT_OK;
}

static inline Result quick_buffer_read_bytes(QuickBuffer* self, void* value, usize len) {
    if(self->capacity - self->pos < len) {
        return RESULT_ERR;
    }

    memcpy(value, (u8*)self->buffer + self->pos, len);
    self->pos += len;

    return RESULT_OK;
}
