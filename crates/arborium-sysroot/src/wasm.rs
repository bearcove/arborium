//! WASM allocator implementation using dlmalloc
//!
//! This module provides C-compatible allocator functions for tree-sitter
//! when compiled to WASM. These functions are exported from the WASM module
//! and used by tree-sitter's C code.

use core::cell::UnsafeCell;
use dlmalloc::Dlmalloc;
use std::ffi::{c_int, c_void};
use std::ptr;

/// Wrapper for Dlmalloc that can be used in static context
///
/// This is safe because WASM is single-threaded.
struct WasmAllocator(UnsafeCell<Dlmalloc>);

// SAFETY: WASM is single-threaded, so we can safely share the allocator.
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

/// Global dlmalloc instance.
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

    let base_ptr = unsafe { user_ptr.sub(HEADER_SIZE) };
    let size = unsafe { ptr::read(base_ptr as *const usize) };
    Some((base_ptr, size))
}

#[inline]
unsafe fn store_size(base_ptr: *mut u8, size: usize) {
    unsafe { ptr::write(base_ptr as *mut usize, size) };
}

/// Allocate memory using dlmalloc.
///
/// # Safety
///
/// Standard malloc unsafety - caller must ensure proper use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }

    let layout = match layout_for_allocation(size) {
        Some(layout) => layout,
        None => return ptr::null_mut(),
    };

    let base_ptr = unsafe { (*ALLOCATOR.get()).malloc(layout.size(), layout.align()) };
    if base_ptr.is_null() {
        return ptr::null_mut();
    }

    unsafe { store_size(base_ptr, size) };
    unsafe { base_ptr.add(HEADER_SIZE) }
}

/// Allocate zeroed memory using dlmalloc.
///
/// # Safety
///
/// Standard calloc unsafety - caller must ensure proper use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    let user_size = match nmemb.checked_mul(size) {
        Some(total) if total != 0 => total,
        _ => return ptr::null_mut(),
    };

    let layout = match layout_for_allocation(user_size) {
        Some(layout) => layout,
        None => return ptr::null_mut(),
    };

    let base_ptr = unsafe { (*ALLOCATOR.get()).calloc(layout.size(), layout.align()) };
    if base_ptr.is_null() {
        return ptr::null_mut();
    }

    unsafe { store_size(base_ptr, user_size) };
    unsafe { base_ptr.add(HEADER_SIZE) }
}

/// Reallocate memory using dlmalloc.
///
/// # Safety
///
/// Standard realloc unsafety - caller must ensure proper use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
    if ptr.is_null() {
        if new_size == 0 {
            return ptr::null_mut();
        }

        let layout = match layout_for_allocation(new_size) {
            Some(layout) => layout,
            None => return ptr::null_mut(),
        };

        let base_ptr = unsafe { (*ALLOCATOR.get()).malloc(layout.size(), layout.align()) };
        if base_ptr.is_null() {
            return ptr::null_mut();
        }

        unsafe { store_size(base_ptr, new_size) };
        return unsafe { base_ptr.add(HEADER_SIZE) };
    }

    if new_size == 0 {
        if let Some((base_ptr, size)) = unsafe { base_ptr_and_size(ptr) } {
            if let Some(layout) = layout_for_allocation(size) {
                unsafe { (*ALLOCATOR.get()).free(base_ptr, layout.size(), layout.align()) };
            }
        }
        return ptr::null_mut();
    }

    let (base_ptr, old_size) = match unsafe { base_ptr_and_size(ptr) } {
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

    let new_ptr = unsafe {
        (*ALLOCATOR.get()).realloc(
            base_ptr,
            old_layout.size(),
            old_layout.align(),
            new_layout.size(),
        )
    };

    if new_ptr.is_null() {
        // Allocation failed, original pointer is still valid.
        return ptr::null_mut();
    }

    unsafe { store_size(new_ptr, new_size) };
    unsafe { new_ptr.add(HEADER_SIZE) }
}

/// Free memory using dlmalloc.
///
/// # Safety
///
/// Standard free unsafety - caller must ensure proper use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }

    if let Some((base_ptr, size)) = unsafe { base_ptr_and_size(ptr) } {
        if let Some(layout) = layout_for_allocation(size) {
            unsafe { (*ALLOCATOR.get()).free(base_ptr, layout.size(), layout.align()) };
        }
    }
}

/// abort implementation - terminates the program.
#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    std::process::abort()
}

/// strncmp implementation - compare two strings up to n bytes.
///
/// # Safety
///
/// Both s1 and s2 must be valid pointers to null-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    if n == 0 {
        return 0;
    }

    for i in 0..n {
        let c1 = unsafe { *s1.add(i) };
        let c2 = unsafe { *s2.add(i) };

        if c1 != c2 {
            return (c1 as c_int) - (c2 as c_int);
        }

        if c1 == 0 {
            return 0;
        }
    }

    0
}

/// strncpy implementation - copy up to n bytes from src to dest.
///
/// # Safety
///
/// Both dest and src must be valid pointers. dest must have space for at least n bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n == 0 {
        return dest;
    }

    let mut i = 0;
    while i < n {
        let c = unsafe { *src.add(i) };
        unsafe { *dest.add(i) = c };

        if c == 0 {
            // Null terminator found, pad the rest with zeros
            i += 1;
            while i < n {
                unsafe { *dest.add(i) = 0 };
                i += 1;
            }
            break;
        }

        i += 1;
    }

    dest
}

/// strcmp implementation - compare two null-terminated strings.
///
/// # Safety
///
/// Both s1 and s2 must be valid pointers to null-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> c_int {
    let mut i = 0;
    loop {
        let c1 = unsafe { *s1.add(i) };
        let c2 = unsafe { *s2.add(i) };

        if c1 != c2 {
            return (c1 as c_int) - (c2 as c_int);
        }

        if c1 == 0 {
            return 0;
        }

        i += 1;
    }
}

/// memchr implementation - locate byte in memory.
///
/// # Safety
///
/// s must be a valid pointer to at least n bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void {
    let s = s as *const u8;
    let c = c as u8;

    for i in 0..n {
        if unsafe { *s.add(i) } == c {
            return unsafe { s.add(i) as *mut c_void };
        }
    }

    ptr::null_mut()
}

/// clock stub - returns 0 for WASM.
#[unsafe(no_mangle)]
pub extern "C" fn clock() -> usize {
    0
}

/// iswspace implementation for wide characters.
#[unsafe(no_mangle)]
pub extern "C" fn iswspace(wc: u32) -> c_int {
    matches!(
        wc,
        0x20
            | 0x09..=0x0D
            | 0xA0
            | 0x1680
            | 0x2000..=0x200A
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
    ) as c_int
}

/// iswalnum implementation for wide characters.
#[unsafe(no_mangle)]
pub extern "C" fn iswalnum(wc: u32) -> c_int {
    (iswalpha(wc) != 0 || iswdigit(wc) != 0) as c_int
}

/// iswdigit implementation for wide characters.
#[unsafe(no_mangle)]
pub extern "C" fn iswdigit(wc: u32) -> c_int {
    matches!(wc, 0x30..=0x39) as c_int
}

/// iswxdigit implementation for wide characters (hex digits: 0-9, A-F, a-f).
#[unsafe(no_mangle)]
pub extern "C" fn iswxdigit(wc: u32) -> c_int {
    (iswdigit(wc) != 0
        || matches!(wc, 0x41..=0x46) // A-F
        || matches!(wc, 0x61..=0x66)) as c_int // a-f
}

/// iswupper implementation - check if wide char is uppercase.
#[unsafe(no_mangle)]
pub extern "C" fn iswupper(wc: u32) -> c_int {
    if (0x41..=0x5A).contains(&wc) {
        return 1;
    }
    if (0xC0..=0xD6).contains(&wc) || (0xD8..=0xDE).contains(&wc) {
        return 1;
    }
    0
}

/// iswlower implementation - check if wide char is lowercase.
#[unsafe(no_mangle)]
pub extern "C" fn iswlower(wc: u32) -> c_int {
    if (0x61..=0x7A).contains(&wc) {
        return 1;
    }
    if (0xE0..=0xF6).contains(&wc) || (0xF8..=0xFF).contains(&wc) {
        return 1;
    }
    0
}

/// iswpunct implementation - check if wide char is punctuation.
#[unsafe(no_mangle)]
pub extern "C" fn iswpunct(wc: u32) -> c_int {
    matches!(wc, 0x21..=0x2F | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E) as c_int
}

/// iswalpha implementation for wide characters.
#[unsafe(no_mangle)]
pub extern "C" fn iswalpha(wc: u32) -> c_int {
    matches!(
        wc,
        0x41..=0x5A
            | 0x61..=0x7A
            | 0xAA
            | 0xB5
            | 0xBA
            | 0xC0..=0xD6
            | 0xD8..=0xF6
            | 0xF8..=0x2C1
            | 0x2C6..=0x2D1
            | 0x2E0..=0x2E4
            | 0x2EC
            | 0x2EE
            | 0x370..=0x374
            | 0x376..=0x377
            | 0x37A..=0x37D
            | 0x37F
            | 0x386
            | 0x388..=0x38A
            | 0x38C
            | 0x38E..=0x3A1
            | 0x3A3..=0x3F5
            | 0x3F7..=0x481
            | 0x48A..=0x52F
            | 0x531..=0x556
            | 0x559
            | 0x560..=0x588
    ) as c_int
}

/// towlower implementation - convert wide char to lowercase.
#[unsafe(no_mangle)]
pub extern "C" fn towlower(wc: u32) -> u32 {
    if (0x41..=0x5A).contains(&wc) {
        return wc + 32;
    }
    if (0xC0..=0xD6).contains(&wc) || (0xD8..=0xDE).contains(&wc) {
        return wc + 32;
    }
    wc
}

/// towupper implementation - convert wide char to uppercase.
#[unsafe(no_mangle)]
pub extern "C" fn towupper(wc: u32) -> u32 {
    if (0x61..=0x7A).contains(&wc) {
        return wc - 32;
    }
    if (0xE0..=0xF6).contains(&wc) || (0xF8..=0xFE).contains(&wc) {
        return wc - 32;
    }
    wc
}

// Force inclusion of allocator / wide-char symbols to prevent dead code elimination.
#[cfg(target_family = "wasm")]
#[used]
static _FORCE_INCLUDE: () = {
    let _ = malloc as *const ();
    let _ = free as *const ();
    let _ = calloc as *const ();
    let _ = realloc as *const ();
    let _ = abort as *const ();
    let _ = strncmp as *const ();
    let _ = strcmp as *const ();
    let _ = strncpy as *const ();
    let _ = memchr as *const ();
    let _ = clock as *const ();
    let _ = iswalnum as *const ();
    let _ = iswalpha as *const ();
    let _ = iswlower as *const ();
    let _ = iswupper as *const ();
    let _ = iswpunct as *const ();
    let _ = iswspace as *const ();
    let _ = iswdigit as *const ();
    let _ = iswxdigit as *const ();
    let _ = towlower as *const ();
    let _ = towupper as *const ();
};
