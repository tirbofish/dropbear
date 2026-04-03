pub mod native {
    pub use eucalyptus_core::scripting::native::DropbearNativeError;
}

pub mod result {
    pub use eucalyptus_core::scripting::result::DropbearNativeResult;
}

pub mod jni {
    pub mod utils {
        pub use crate::FromJObject;
        pub use crate::ToJObject;
    }
}
