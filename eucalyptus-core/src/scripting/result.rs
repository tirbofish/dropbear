use crate::scripting::native::DropbearNativeError;

/// A result type for dropbear based native functions.
pub type DropbearNativeResult<T> = Result<T, DropbearNativeError>;