use std::path::Path;

pub struct CompiledSlangShader {
    pub(crate) label: String,
    args: String,
    pub source: Vec<u8>,
}

impl CompiledSlangShader {
    /// Internal: Creates a new instance of [CompiledSlangShader] from the compiled args
    /// of [crate::SlangShaderBuilder]
    pub(crate) fn new(label: String, args: String, source: Vec<u8>) -> Self {
        Self {
            args,
            source,
            label,
        }
    }

    /// Fetches the command line arguments provided into [crate::SlangShaderBuilder]
    pub fn args(&self) -> String {
        self.args.clone()
    }

    /// Creates a [CompiledSlangShader] from raw bytes.
    ///
    /// This is useful when loading shaders compiled at build time using the
    /// `include_slang!` macro.
    pub fn from_bytes(label: &str, source: &[u8]) -> Self {
        Self {
            label: label.to_string(),
            args: String::new(),
            source: source.to_vec(),
        }
    }

    /// Returns the label of the shader.
    pub fn label(&self) -> String {
        self.label.clone()
    }

    /// Writes the output to a file.
    pub fn output(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();

        std::fs::write(path, &self.source)
    }
}
