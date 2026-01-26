use std::path::Path;

pub struct CompiledSlangShader {
    args: String,
    pub source: Vec<u8>,
}

impl CompiledSlangShader {
    /// Internal: Creates a new instance of [CompiledSlangShader] from the compiled args
    /// of [crate::SlangShaderBuilder]
    pub(crate) fn new(args: String, source: Vec<u8>) -> Self {
        Self {
            args,
            source
        }
    }

    /// Fetches the command line arguments provided into [crate::SlangShaderBuilder]
    pub fn args(&self) -> String {
        self.args.clone()
    }

    /// Writes the output to a file.
    pub fn output(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();

        std::fs::write(path, &self.source)
    }
}