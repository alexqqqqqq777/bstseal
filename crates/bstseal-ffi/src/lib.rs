//! C-compatible FFI for BST-SEAL.
//!
//! Safety contract:
//! • All functions are `extern "C"`.
//! • Heap memory is (de)allocated via the system allocator (`libc::malloc/free`).
//! • On success return 0, on failure non-zero (see `ErrorCode`).
//! • Caller must free returned buffers with `bstseal_free`.

use std::slice;
use libc::{c_int, c_void, malloc, free};
use bstseal_core::{encode::{encode_parallel, decode_parallel}, integrity};

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum ErrorCode {
    Ok = 0,
    NullPointer = 1,
    EncodeFail = 2,
    DecodeFail = 3,
    IntegrityFail = 4,
    AllocFail = 5,
}

unsafe fn alloc(len: usize) -> *mut u8 {
    let ptr = malloc(len) as *mut u8;
    if ptr.is_null() { std::ptr::null_mut() } else { ptr }
}

#[no_mangle]
pub unsafe extern "C" fn bstseal_encode(
    input: *const u8,
    len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if input.is_null() || out_ptr.is_null() || out_len.is_null() { return ErrorCode::NullPointer as c_int; }
    let data = slice::from_raw_parts(input, len);
    let compressed = match encode_parallel(data) {
        Ok(c) => c,
        Err(_) => return ErrorCode::EncodeFail as c_int,
    };
    let with_footer = integrity::add_footer(&compressed);
    let buf = alloc(with_footer.len());
    if buf.is_null() { return ErrorCode::AllocFail as c_int; }
    std::ptr::copy_nonoverlapping(with_footer.as_ptr(), buf, with_footer.len());
    *out_ptr = buf;
    *out_len = with_footer.len();
    ErrorCode::Ok as c_int
}

#[no_mangle]
pub unsafe extern "C" fn bstseal_decode(
    input: *const u8,
    len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if input.is_null() || out_ptr.is_null() || out_len.is_null() { return ErrorCode::NullPointer as c_int; }
    let data = slice::from_raw_parts(input, len);
    let payload = match integrity::verify_footer(data) {
        Ok(p) => p,
        Err(_) => return ErrorCode::IntegrityFail as c_int,
    };
    let decoded = match decode_parallel(payload) {
        Ok(d) => d,
        Err(_) => return ErrorCode::DecodeFail as c_int,
    };
    let buf = alloc(decoded.len());
    if buf.is_null() { return ErrorCode::AllocFail as c_int; }
    std::ptr::copy_nonoverlapping(decoded.as_ptr(), buf, decoded.len());
    *out_ptr = buf;
    *out_len = decoded.len();
    ErrorCode::Ok as c_int
}

#[no_mangle]
pub unsafe extern "C" fn bstseal_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        free(ptr);
    }
}
