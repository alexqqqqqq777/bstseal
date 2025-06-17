//! C-compatible FFI for BST-SEAL.
//!
//! Safety contract:
//! • All functions are `extern "C"`.
//! • Heap memory is (de)allocated via the system allocator (`libc::malloc/free`).
//! • On success return 0, on failure non-zero (see `ErrorCode`).
//! • Caller must free returned buffers with `bstseal_free`.

use bstseal_core::{
    encode::{decode_parallel, encode_parallel},
    integrity,
};
use libc::{c_int, c_void, c_char, free, malloc};
use std::slice;

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum ErrorCode {
    Ok = 0,
    NullPointer = 1,
    EncodeFail = 2,
    DecodeFail = 3,
    IntegrityFail = 4,
    AllocFail = 5,
    LicenseError = 6,
}

unsafe fn alloc(len: usize) -> *mut u8 {
    let ptr = malloc(len) as *mut u8;
    if ptr.is_null() {
        std::ptr::null_mut()
    } else {
        ptr
    }
}

#[no_mangle]
/// Compresses `input` and returns a newly allocated buffer containing
/// the encoded bytes **plus integrity footer**.
///
/// On success returns [`ErrorCode::Ok`] (0) and sets `out_ptr` / `out_len`.
///
/// # Safety
/// * `input` must point to `len` valid bytes.
/// * `out_ptr` and `out_len` must be valid, non-null pointers.
/// * Caller owns the returned buffer and must free it with [`bstseal_free`].
pub unsafe extern "C" fn bstseal_encode(
    input: *const u8,
    len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if input.is_null() || out_ptr.is_null() || out_len.is_null() {
        return ErrorCode::NullPointer as c_int;
    }
    let data = slice::from_raw_parts(input, len);
    let compressed = match encode_parallel(data) {
        Ok(c) => c,
        Err(_) => return ErrorCode::EncodeFail as c_int,
    };
    let with_footer = integrity::add_footer(&compressed);
    let buf = alloc(with_footer.len());
    if buf.is_null() {
        return ErrorCode::AllocFail as c_int;
    }
    std::ptr::copy_nonoverlapping(with_footer.as_ptr(), buf, with_footer.len());
    *out_ptr = buf;
    *out_len = with_footer.len();
    ErrorCode::Ok as c_int
}

#[no_mangle]
/// Verifies integrity footer and decompresses `input`.
///
/// On success returns [`ErrorCode::Ok`] and sets `out_ptr` / `out_len`.
///
/// # Safety
/// * `input` must point to `len` valid bytes produced by [`bstseal_encode`].
/// * `out_ptr` and `out_len` must be valid, non-null pointers.
/// * Caller owns the returned buffer and must free it with [`bstseal_free`].
pub unsafe extern "C" fn bstseal_decode(
    input: *const u8,
    len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if input.is_null() || out_ptr.is_null() || out_len.is_null() {
        return ErrorCode::NullPointer as c_int;
    }
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
    if buf.is_null() {
        return ErrorCode::AllocFail as c_int;
    }
    std::ptr::copy_nonoverlapping(decoded.as_ptr(), buf, decoded.len());
    *out_ptr = buf;
    *out_len = decoded.len();
    ErrorCode::Ok as c_int
}

#[no_mangle]
/// Frees a buffer allocated by [`bstseal_encode`] / [`bstseal_decode`].
///
/// # Safety
/// * `ptr` must be a pointer previously obtained from those functions (or null).
pub unsafe extern "C" fn bstseal_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        free(ptr);
    }
}

#[no_mangle]
/// Sets license secret at runtime.
/// Returns 0 on success.
/// # Safety
/// * `secret` must be a valid null-terminated UTF-8 string or NULL.
pub unsafe extern "C" fn bstseal_set_license_secret(secret: *const c_char) -> c_int {
    if secret.is_null() {
        return ErrorCode::NullPointer as c_int;
    }
    let c_str = std::ffi::CStr::from_ptr(secret);
    match c_str.to_str() {
        Ok(s) => {
            bstseal_core::license::set_license_secret(s.to_string());
            ErrorCode::Ok as c_int
        }
        Err(_) => ErrorCode::LicenseError as c_int,
    }
}

#[no_mangle]
/// Sets license key at runtime.
/// Returns 0 on success.
/// # Safety
/// * `key` must be a valid null-terminated UTF-8 string or NULL.
pub unsafe extern "C" fn bstseal_set_license_key(key: *const c_char) -> c_int {
    if key.is_null() {
        return ErrorCode::NullPointer as c_int;
    }
    let c_str = std::ffi::CStr::from_ptr(key);
    match c_str.to_str() {
        Ok(k) => {
            bstseal_core::license::set_license_key(k.to_string());
            ErrorCode::Ok as c_int
        }
        Err(_) => ErrorCode::LicenseError as c_int,
    }
}
