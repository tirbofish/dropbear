package com.dropbear.ffi;

/// Interface for dynamic library loading, allowing for custom components not within the dropbear component
/// library.
public interface DynamicLibraryLoader {
    /// Ensures that the library is loaded before accessing
    /// any of the JNI static functions.
    void ensureLoaded();

    /// Forces the library to have a loaded boolean variable.
    boolean isLoaded();
}
