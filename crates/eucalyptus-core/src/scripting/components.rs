use crate::scripting::ScriptManager;

/// As the name suggests, it's just callbacks defined from another source for
/// Component based functions.
///
/// I need to get a better name
pub enum ComponentDefinedSomewhereElseCallbacks {
    /// Defines a JVM based interface, which uses the [jni] crate for its backend implementation.
    JVM {},

    /// Defines a C-based interface, such as Kotlin/Native or another c-based language.
    Native {},
}

mod jvm {
    use crate::scripting::jni::JavaContext;

    impl JavaContext {
        pub fn register_components() -> anyhow::Result<()> {
            Ok(())
        }

        pub fn register_callbacks() -> anyhow::Result<()> {
            Ok(())
        }
    }
}

mod native {
    use crate::scripting::native::NativeLibrary;

    impl NativeLibrary {
        pub fn register_components() -> anyhow::Result<()> {
            Ok(())
        }

        pub fn register_callbacks() -> anyhow::Result<()> {
            Ok(())
        }
    }
}

impl ScriptManager {
    pub fn register_components() -> anyhow::Result<()> {
        Ok(())
    }

    pub fn register_callbacks() -> anyhow::Result<()> {
        Ok(())
    }
}
