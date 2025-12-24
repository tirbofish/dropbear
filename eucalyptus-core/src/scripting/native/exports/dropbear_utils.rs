use std::ffi::{c_char};

/// A double-precision floating point number. Convenient!
pub type Double = f64;

/// The sister to [`crate::utils::Progress`], which provides C-compatible values.
#[repr(C)]
pub struct Progress {
    pub current: Double,
    pub total: Double,
    pub message: *mut c_char,
}