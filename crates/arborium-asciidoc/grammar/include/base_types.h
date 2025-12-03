#pragma once
#include "../tree_sitter/parser.h"

#ifndef u8
typedef uint8_t u8;
#endif

#ifndef i8
typedef int8_t i8;
#endif

#ifndef u32
typedef uint32_t u32;
#endif

#ifndef i32
typedef int32_t i32;
#endif

#ifndef usize
typedef uintptr_t usize;
#endif

#ifndef isize
typedef intptr_t isize;
#endif

#ifndef USIZE_MAX
static const usize USIZE_MAX = UINTPTR_MAX;
#endif

#ifndef ISIZE_MAX
static const isize ISIZE_MAX = INTPTR_MAX;
#endif

#define ADOC_UNUSED(x) ((void)x)
