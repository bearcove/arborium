//! WASM allocator implementation using dlmalloc
//!
//! This module provides C-compatible allocator functions for tree-sitter
//! when compiled to WASM. These functions are exported from the WASM module
//! and used by tree-sitter's C code.
//!
//! # The Problem
//!
//! Tree-sitter is written in C and compiled to WASM. When tree-sitter's C code
//! needs to allocate memory, it calls standard C functions like `malloc`, `free`, etc.
//!
//! In a WASM environment, these functions need to be provided by the host. By default,
//! tree-sitter expects them to be available in the "env" module, which causes this error:
//!
//! ```text
//! Uncaught TypeError: Failed to resolve module specifier "env".
//! Relative references must start with either "/", "./", or "../".
//! ```
//!
//! # The Solution
//!
//! This module provides WASM allocator functions (`malloc`, `calloc`, `realloc`, `free`)
//! that are exported from the WASM module. These functions use dlmalloc for actual
//! memory management.
//!
//! # Usage
//!
//! This module is automatically enabled when compiling for WASM targets.
//! You don't need to call these functions directly - having this module
//! compiled in ensures the symbols are exported.
//!
//! # Debugging
//!
//! To verify the allocator is working, check that these symbols are exported in your WASM:
//!
//! ```bash
//! wasm-objdump -x your_app_bg.wasm | grep -E "(malloc|calloc|realloc|free)"
//! ```

use core::cell::UnsafeCell;
use dlmalloc::Dlmalloc;
use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Wrapper for Dlmalloc that can be used in static context
///
/// This is safe because WASM is single-threaded
struct WasmAllocator(UnsafeCell<Dlmalloc>);

// SAFETY: WASM is single-threaded, so we can safely share the allocator
unsafe impl Sync for WasmAllocator {}

impl WasmAllocator {
    const fn new() -> Self {
        WasmAllocator(UnsafeCell::new(Dlmalloc::new()))
    }

    #[inline]
    fn get(&self) -> *mut Dlmalloc {
        self.0.get()
    }
}

/// Global dlmalloc instance
static ALLOCATOR: WasmAllocator = WasmAllocator::new();

const ALIGNMENT: usize = std::mem::size_of::<usize>();
const HEADER_SIZE: usize = std::mem::size_of::<usize>();

#[inline]
fn layout_for_allocation(size: usize) -> Option<std::alloc::Layout> {
    size.checked_add(HEADER_SIZE)
        .and_then(|total| std::alloc::Layout::from_size_align(total, ALIGNMENT).ok())
}

#[inline]
unsafe fn base_ptr_and_size(user_ptr: *mut u8) -> Option<(*mut u8, usize)> {
    if user_ptr.is_null() {
        return None;
    }

    unsafe {
        let base_ptr = user_ptr.sub(HEADER_SIZE);
        let size = ptr::read(base_ptr as *const usize);
        Some((base_ptr, size))
    }
}

#[inline]
unsafe fn store_size(base_ptr: *mut u8, size: usize) {
    unsafe {
        ptr::write(base_ptr as *mut usize, size);
    }
}

/// Allocate memory using dlmalloc
///
/// # Safety
///
/// Standard malloc unsafety - caller must ensure proper use
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }

    unsafe {
        let layout = match layout_for_allocation(size) {
            Some(layout) => layout,
            None => return ptr::null_mut(),
        };
        let base_ptr = (*ALLOCATOR.get()).malloc(layout.size(), layout.align());
        if base_ptr.is_null() {
            return ptr::null_mut();
        }
        store_size(base_ptr, size);
        base_ptr.add(HEADER_SIZE)
    }
}

/// Allocate zeroed memory using dlmalloc
///
/// # Safety
///
/// Standard calloc unsafety - caller must ensure proper use
#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    let user_size = match nmemb.checked_mul(size) {
        Some(total) if total != 0 => total,
        _ => return ptr::null_mut(),
    };

    unsafe {
        let layout = match layout_for_allocation(user_size) {
            Some(layout) => layout,
            None => return ptr::null_mut(),
        };
        let base_ptr = (*ALLOCATOR.get()).calloc(layout.size(), layout.align());
        if base_ptr.is_null() {
            return ptr::null_mut();
        }
        store_size(base_ptr, user_size);
        let user_ptr = base_ptr.add(HEADER_SIZE);
        user_ptr
    }
}

/// Reallocate memory using dlmalloc
///
/// # Safety
///
/// Standard realloc unsafety - caller must ensure proper use
#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
    if ptr.is_null() {
        // Just allocate new memory
        if new_size == 0 {
            return ptr::null_mut();
        }
        unsafe {
            let layout = match layout_for_allocation(new_size) {
                Some(layout) => layout,
                None => return ptr::null_mut(),
            };
            let base_ptr = (*ALLOCATOR.get()).malloc(layout.size(), layout.align());
            if base_ptr.is_null() {
                return ptr::null_mut();
            }
            store_size(base_ptr, new_size);
            return base_ptr.add(HEADER_SIZE);
        }
    }

    if new_size == 0 {
        // Free the memory
        unsafe {
            if let Some((base_ptr, size)) = base_ptr_and_size(ptr) {
                if let Some(layout) = layout_for_allocation(size) {
                    (*ALLOCATOR.get()).free(base_ptr, layout.size(), layout.align());
                }
            }
        }
        return ptr::null_mut();
    }

    unsafe {
        let (base_ptr, old_size) = match base_ptr_and_size(ptr) {
            Some(values) => values,
            None => return ptr::null_mut(),
        };

        let old_layout = match layout_for_allocation(old_size) {
            Some(layout) => layout,
            None => return ptr::null_mut(),
        };

        let new_layout = match layout_for_allocation(new_size) {
            Some(layout) => layout,
            None => return ptr::null_mut(),
        };

        let new_ptr = (*ALLOCATOR.get()).realloc(
            base_ptr,
            old_layout.size(),
            old_layout.align(),
            new_layout.size(),
        );
        if new_ptr.is_null() {
            // Allocation failed, original pointer is still valid
            return ptr::null_mut();
        }

        store_size(new_ptr, new_size);
        new_ptr.add(HEADER_SIZE)
    }
}

/// Free memory using dlmalloc
///
/// # Safety
///
/// Standard free unsafety - caller must ensure proper use
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        if let Some((base_ptr, size)) = base_ptr_and_size(ptr) {
            if let Some(layout) = layout_for_allocation(size) {
                (*ALLOCATOR.get()).free(base_ptr, layout.size(), layout.align());
            }
        }
    }
}

// C standard library stubs needed by tree-sitter

/// strncmp implementation for comparing C strings
///
/// # Safety
///
/// Caller must ensure s1 and s2 are valid pointers to null-terminated strings
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }

    unsafe {
        for i in 0..n {
            let c1 = *s1.add(i) as u8;
            let c2 = *s2.add(i) as u8;

            if c1 == 0 || c2 == 0 {
                return (c1 as i32) - (c2 as i32);
            }

            if c1 != c2 {
                return (c1 as i32) - (c2 as i32);
            }
        }
    }

    0
}

/// strcmp implementation for comparing C strings
///
/// # Safety
///
/// Caller must ensure s1 and s2 are valid pointers to null-terminated strings
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }

    unsafe {
        let mut i = 0;
        loop {
            let c1 = *s1.add(i) as u8;
            let c2 = *s2.add(i) as u8;

            if c1 == 0 || c2 == 0 || c1 != c2 {
                return (c1 as i32) - (c2 as i32);
            }

            i += 1;
        }
    }
}

/// strncpy implementation for copying C strings with length limit
///
/// # Safety
///
/// Caller must ensure dest has space for n bytes and src is valid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char {
    unsafe {
        for i in 0..n {
            let c = *src.add(i);
            *dest.add(i) = c;
            if c == 0 {
                break;
            }
        }
    }
    dest
}

/// strchr implementation for finding a character in a string
///
/// # Safety
///
/// Caller must ensure s is a valid pointer to a null-terminated string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let mut p = s;
        let target = c as u8;
        loop {
            let current = *p as u8;
            if current == target {
                return p as *mut c_char;
            }
            if current == 0 {
                return ptr::null_mut();
            }
            p = p.add(1);
        }
    }
}

/// memchr implementation for finding a byte in a memory region
///
/// # Safety
///
/// Caller must ensure s is a valid pointer to at least n bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void {
    if s.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let p = s as *const u8;
        let target = c as u8;
        for i in 0..n {
            if *p.add(i) == target {
                return p.add(i) as *mut c_void;
            }
        }
    }
    ptr::null_mut()
}

/// fclose stub - no-op for WASM
#[unsafe(no_mangle)]
pub extern "C" fn fclose(_stream: *mut c_void) -> c_int {
    0
}

/// fdopen stub - returns null for WASM
#[unsafe(no_mangle)]
pub extern "C" fn fdopen(_fd: c_int, _mode: *const c_char) -> *mut c_void {
    ptr::null_mut()
}

/// clock stub - returns 0 for WASM
#[unsafe(no_mangle)]
pub extern "C" fn clock() -> usize {
    0
}

/// fwrite stub - no-op for WASM
#[unsafe(no_mangle)]
pub extern "C" fn fwrite(
    _ptr: *const c_void,
    _size: usize,
    _nmemb: usize,
    _stream: *mut c_void,
) -> usize {
    0
}

/// fputc stub - no-op for WASM
#[unsafe(no_mangle)]
pub extern "C" fn fputc(_c: c_int, _stream: *mut c_void) -> c_int {
    0
}

/// iswspace implementation for wide characters
#[unsafe(no_mangle)]
pub extern "C" fn iswspace(wc: u32) -> c_int {
    matches!(
        wc,
        0x20 | 0x09..=0x0D | 0xA0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000
    ) as c_int
}

/// iswalnum implementation for wide characters
#[unsafe(no_mangle)]
pub extern "C" fn iswalnum(wc: u32) -> c_int {
    (iswalpha(wc) != 0 || iswdigit(wc) != 0) as c_int
}

/// iswdigit implementation for wide characters
#[unsafe(no_mangle)]
pub extern "C" fn iswdigit(wc: u32) -> c_int {
    matches!(wc, 0x30..=0x39) as c_int
}

/// iswxdigit implementation for wide characters (hex digits: 0-9, A-F, a-f)
#[unsafe(no_mangle)]
pub extern "C" fn iswxdigit(wc: u32) -> c_int {
    matches!(wc, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66) as c_int
}

/// iswalpha implementation for wide characters
#[unsafe(no_mangle)]
pub extern "C" fn iswalpha(wc: u32) -> c_int {
    matches!(
        wc,
        0x41..=0x5A | 0x61..=0x7A | 0xAA | 0xB5 | 0xBA | 0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0x2C1 |
        0x2C6..=0x2D1 | 0x2E0..=0x2E4 | 0x2EC | 0x2EE | 0x370..=0x374 | 0x376..=0x377 |
        0x37A..=0x37D | 0x37F | 0x386 | 0x388..=0x38A | 0x38C | 0x38E..=0x3A1 | 0x3A3..=0x3F5 |
        0x3F7..=0x481 | 0x48A..=0x52F | 0x531..=0x556 | 0x559 | 0x560..=0x588
    ) as c_int
}

/// iswupper implementation - check if wide char is uppercase
#[unsafe(no_mangle)]
pub extern "C" fn iswupper(wc: u32) -> c_int {
    // Basic Latin uppercase A-Z
    if (0x41..=0x5A).contains(&wc) {
        return 1;
    }
    // Latin-1 Supplement uppercase (À-Ö, Ø-Þ)
    if (0xC0..=0xD6).contains(&wc) || (0xD8..=0xDE).contains(&wc) {
        return 1;
    }
    0
}

/// iswlower implementation - check if wide char is lowercase
#[unsafe(no_mangle)]
pub extern "C" fn iswlower(wc: u32) -> c_int {
    // Basic Latin lowercase a-z
    if (0x61..=0x7A).contains(&wc) {
        return 1;
    }
    // Latin-1 Supplement lowercase (à-ö, ø-þ)
    if (0xE0..=0xF6).contains(&wc) || (0xF8..=0xFE).contains(&wc) {
        return 1;
    }
    0
}

/// towlower implementation - convert wide char to lowercase
#[unsafe(no_mangle)]
pub extern "C" fn towlower(wc: u32) -> u32 {
    // Basic Latin uppercase A-Z
    if (0x41..=0x5A).contains(&wc) {
        return wc + 32;
    }
    // Latin-1 Supplement uppercase (À-Ö, Ø-Þ)
    if (0xC0..=0xD6).contains(&wc) || (0xD8..=0xDE).contains(&wc) {
        return wc + 32;
    }
    wc
}

/// towupper implementation - convert wide char to uppercase
#[unsafe(no_mangle)]
pub extern "C" fn towupper(wc: u32) -> u32 {
    // Basic Latin lowercase a-z
    if (0x61..=0x7A).contains(&wc) {
        return wc - 32;
    }
    // Latin-1 Supplement lowercase (à-ö, ø-þ)
    if (0xE0..=0xF6).contains(&wc) || (0xF8..=0xFE).contains(&wc) {
        return wc - 32;
    }
    wc
}

/// fputs stub - no-op for WASM
#[unsafe(no_mangle)]
pub extern "C" fn fputs(_s: *const c_char, _stream: *mut c_void) -> c_int {
    0
}

/// abort stub - no-op for WASM
#[unsafe(no_mangle)]
pub extern "C" fn abort() {
    // No-op for WASM - can't really abort in this context
}

/// dup stub - duplicate file descriptor (returns -1 for unsupported in WASM)
#[unsafe(no_mangle)]
pub extern "C" fn dup(_fd: c_int) -> c_int {
    -1 // Return error since file descriptors aren't supported in WASM
}
