#pragma once

#include "base_types.h"
#include "quick_buffer.h"

typedef enum BlockKind {
    BLOCK_KIND_DELIMITED,
    BLOCK_KIND_TABLE,
    BLOCK_KIND_LISTING,
    BLOCK_KIND_LITERAL
} BlockKind;

typedef struct Node {
    BlockKind kind;
    usize counter;
} Node;

typedef struct Scanner {
    usize capacity;
    usize len;
    Node *buffer;
} Scanner;

static inline void scanner_init(Scanner *self);
static inline void scanner_free(Scanner *self);

static inline Node *scanner_top(Scanner const *self);
static inline bool scanner_is_matching_raw_block(Scanner *self);
static inline bool scanner_is_expect_block_start(Scanner const *self);
/**
 * @brief check scanner is matching a block
 *
 * @param self Scanner
 * @param kind BlockKind
 * @param counter block counter, zero means any counter.
 * @return true if matched.
 */
static inline bool scanner_is_matching(Scanner const *self, BlockKind kind, usize counter);
static inline Result scanner_serialize(Scanner const *self, QuickBuffer *qb);
static inline Result scanner_deserialize(Scanner *self, QuickBuffer *qb);
/**
 * @brief pop if scanner_is_matching block
 *
 * @param self Scanner
 * @param kind BlockKind
 * @param counter block counter, zero means any counter.
 * @return true if pop a block.
 */
static inline bool scanner_pop_kind(Scanner *self, BlockKind kind, usize counter);
static inline void scanner_pop(Scanner *self);
static inline void scanner_push(Scanner *self, BlockKind kind, usize counter);
