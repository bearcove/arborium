#include "../tree_sitter/parser.h"

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
const usize USIZE_MAX = UINTPTR_MAX;
#endif

#ifndef ISIZE_MAX
const isize ISIZE_MAX = INTPTR_MAX;
#endif
