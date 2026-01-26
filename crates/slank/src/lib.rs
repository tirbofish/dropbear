//! slank - slangc for rust. 
//! 
//! Compiles slang code during build and stores the shaders locally (or in the crate with [`include_slang`])
//!
//! Check out [`SlangShaderBuilder`] to get started.

/// Fetches the slang file (located in {OUT_DIR}/{label}.spv) (assuming it is compiled as .spv) 
/// and includes the bytes of the file. 
#[macro_export]
macro_rules! include_slang {
    ($label:expr) => {
        include_bytes!(concat!(env!("OUT_DIR"), "/", $label, ".spv"))
    };
}

/// Fetches the path of the shader (with the same label) and returns it to you. 
#[macro_export]
macro_rules! include_slang_path {
    ($label:expr) => {
        concat!(env!("OUT_DIR"), "/", $label, ".spv")
    };
}

pub mod compiled;
pub mod utils;

use std::{fmt::Display, path::{Path, PathBuf}};
use crate::compiled::CompiledSlangShader;

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: Option<PathBuf>,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct EntryPoint {
    pub name: String,
    pub stage: Option<ShaderStage>,
    /// Index into the sources vec - which file this entry point comes from
    pub source_index: Option<usize>,
}

/// Allows you to pass arguments and make an argument builder to chuck into the slangc
/// compiler.
///
/// This is the entry point of the library.
/// # Usage
/// 
/// Add `slank` to your `[build-dependencies]`.
///
/// In your `build.rs`:
/// ```rust,no_run
/// use slank::{ShaderStage, SlangShaderBuilder, SlangTarget};
/// use std::path::Path;
///
/// fn main() {
///     let out_dir = std::env::var("OUT_DIR").unwrap();
///     let dest_path = Path::new(&out_dir).join("shader.spv");
///
///     SlangShaderBuilder::new("shader_label")
///         .add_source_path("src/shader.slang").unwrap()
///         .entry_with_stage("vs_main", ShaderStage::Vertex)
///         .entry_with_stage("fs_main", ShaderStage::Fragment)
///         .build(SlangTarget::SpirV).unwrap()
///         .output(&dest_path).unwrap();
/// 
///     println!("cargo:rerun-if-changed=src/shader.slang");
/// }
/// ```
///
/// Then in your main code:
/// ```rust,ignore
/// let shader_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/shader.spv"));
/// ```
pub struct SlangShaderBuilder {
    label: String,
    sources: Vec<SourceFile>,
    entries: Vec<EntryPoint>,
    profile: Option<Profile>,
    additional_args: Vec<String>,
}

impl SlangShaderBuilder {
    /// Creates a new instance of a [SlangShaderBuilder].
    pub fn new(label: &str) -> Self {
        Self {
            sources: Vec::new(),
            entries: Vec::new(),
            label: label.to_string(),
            profile: None,
            additional_args: Vec::new(),
        }
    }

    /// Adds a source file by path.
    ///
    /// Returns the index of this source, which can be used with
    /// `entry_from_source()` to associate entry points with specific files.
    pub fn add_source_path(mut self, path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path_buf = path.as_ref().to_path_buf();
        let content = std::fs::read_to_string(&path_buf)?;

        self.sources.push(SourceFile {
            path: Some(path_buf),
            content,
        });

        Ok(self)
    }

    /// Adds a source file by path (non-consuming version).
    pub fn source_path(&mut self, path: impl AsRef<Path>) -> Result<usize, std::io::Error> {
        let path_buf = path.as_ref().to_path_buf();
        let content = std::fs::read_to_string(&path_buf)?;

        self.sources.push(SourceFile {
            path: Some(path_buf),
            content,
        });

        Ok(self.sources.len() - 1)
    }

    /// Adds source code as a string.
    ///
    /// Returns the index of this source.
    pub fn add_source_str(mut self, content: &str) -> Self {
        self.sources.push(SourceFile {
            path: None,
            content: content.to_string(),
        });
        self
    }

    /// Adds source code as a string (non-consuming version).
    pub fn source_str(&mut self, content: &str) -> usize {
        self.sources.push(SourceFile {
            path: None,
            content: content.to_string(),
        });
        self.sources.len() - 1
    }

    /// Adds multiple source files at once.
    pub fn add_source_paths(mut self, paths: &[impl AsRef<Path>]) -> Result<Self, std::io::Error> {
        for path in paths {
            let path_buf = path.as_ref().to_path_buf();
            let content = std::fs::read_to_string(&path_buf)?;

            self.sources.push(SourceFile {
                path: Some(path_buf),
                content,
            });
        }
        Ok(self)
    }

    /// Adds an entry point from the most recently added source.
    ///
    /// According to Slang docs: "the file associated with the entry point
    /// will be the first one found when searching to the left in the command line."
    pub fn entry(mut self, name: &str) -> Self {
        let source_index = if self.sources.is_empty() {
            None
        } else {
            Some(self.sources.len() - 1)
        };

        self.entries.push(EntryPoint {
            name: name.to_string(),
            stage: None,
            source_index,
        });
        self
    }

    /// Adds an entry point with an explicit shader stage.
    pub fn entry_with_stage(mut self, name: &str, stage: ShaderStage) -> Self {
        let source_index = if self.sources.is_empty() {
            None
        } else {
            Some(self.sources.len() - 1)
        };

        self.entries.push(EntryPoint {
            name: name.to_string(),
            stage: Some(stage),
            source_index,
        });
        self
    }

    /// Adds an entry point from a specific source file (by index).
    pub fn entry_from_source(mut self, name: &str, source_index: usize) -> Self {
        self.entries.push(EntryPoint {
            name: name.to_string(),
            stage: None,
            source_index: Some(source_index),
        });
        self
    }

    /// Adds an entry point with stage from a specific source file.
    pub fn entry_from_source_with_stage(
        mut self,
        name: &str,
        source_index: usize,
        stage: ShaderStage
    ) -> Self {
        self.entries.push(EntryPoint {
            name: name.to_string(),
            stage: Some(stage),
            source_index: Some(source_index),
        });
        self
    }

    pub fn with_profile(mut self, profile: Profile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// In the case that there was an argument not available to this builder, you can
    /// manually provide it here. 
    pub fn with_additional_args(mut self, args: &[&str]) -> Self {
        self.additional_args = args.to_vec().iter().map(|v| v.to_string()).collect::<Vec<_>>();
        self
    }

    pub fn compile_to_out_dir(self, target: SlangTarget) -> anyhow::Result<()> {
        let label = self.label.clone();
        let compiled = self.build(target)?;
        let out_dir = std::env::var("OUT_DIR")
            .map_err(|_| anyhow::anyhow!("OUT_DIR not set. Are you running this from build.rs?"))?;
        let dest = Path::new(&out_dir).join(format!("{}.spv", label));
        compiled.output(&dest).map_err(Into::into)
    }

    pub fn build(self, target: SlangTarget) -> anyhow::Result<CompiledSlangShader> {
        let slang_dir = PathBuf::from(env!("SLANG_DIR"));
        let slangc_path = slang_dir.join("bin").join(if cfg!(windows) { "slangc.exe" } else { "slangc" });

        if !slangc_path.exists() {
            anyhow::bail!("slangc executable not found at {}", slangc_path.display());
        }

        let mut cmd = std::process::Command::new(slangc_path);
        let mut args_record = String::from("slangc");

        let mut temp_files = Vec::new();

        cmd.arg("-target").arg(target.as_arg());

        use std::fmt::Write;
        write!(&mut args_record, " -target {}", target.as_arg())?;

        for (source_idx, source) in self.sources.iter().enumerate() {
            let path_to_use = if let Some(path) = &source.path {
                path.clone()
            } else {
                let mut temp = tempfile::Builder::new()
                    .suffix(".slang")
                    .tempfile()?;

                use std::io::Write as IoWrite;
                temp.write_all(source.content.as_bytes())?;

                let p = temp.path().to_path_buf();
                temp_files.push(temp);
                p
            };

            cmd.arg(&path_to_use);
            write!(&mut args_record, " {}", path_to_use.display())?;

            for entry in &self.entries {
                if entry.source_index == Some(source_idx) {
                    cmd.arg("-entry").arg(&entry.name);
                    write!(&mut args_record, " -entry {}", entry.name)?;

                    if let Some(stage) = entry.stage {
                        cmd.arg("-stage").arg(stage.as_arg());
                        write!(&mut args_record, " -stage {}", stage.as_arg())?;
                    }
                }
            }
        }

        cmd.args(&self.additional_args);
        write!(&mut args_record, " {}", self.additional_args.join(" "))?;

        let temp_dir = tempfile::tempdir()?;
        let output_path = temp_dir.path().join("output");

        cmd.arg("-o").arg(&output_path);
        write!(&mut args_record, " -o {}", output_path.display())?;
        println!("cmd: {:?}", cmd.get_args());
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(anyhow::anyhow!("Compilation error: {}", stderr));
        }


        let binary_output = std::fs::read(&output_path)?;

        Ok(CompiledSlangShader::new(self.label, args_record, binary_output))
    }
}

impl Default for SlangShaderBuilder {
    fn default() -> Self {
        Self::new("no named shader")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    // wgpu doesnt support anything below this
    Geometry,
    Hull,
    Domain,
    RayGen,
    Intersection,
    AnyHit,
    ClosestHit,
    Miss,
    Callable,
}

impl ShaderStage {
    /// Convert to the slangc command-line argument
    pub fn as_arg(&self) -> &'static str {
        match self {
            Self::Vertex => "vertex",
            Self::Fragment => "fragment",
            Self::Compute => "compute",
            Self::Geometry => "geometry",
            Self::Hull => "hull",
            Self::Domain => "domain",
            Self::RayGen => "raygeneration",
            Self::Intersection => "intersection",
            Self::AnyHit => "anyhit",
            Self::ClosestHit => "closesthit",
            Self::Miss => "miss",
            Self::Callable => "callable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlangTarget {
    Unknown,
    None,

    // HLSL/DirectX
    Hlsl,
    Dxbc,
    DxbcAssembly,
    Dxil,
    DxilAssembly,

    // GLSL/Vulkan/SPIR-V
    Glsl,
    /// Most optimal for WGPU and Vulkan/ 
    SpirV,
    SpirVAssembly,

    // C/C++
    C,
    Cpp,
    CppHeader,
    TorchBinding,
    HostCpp,

    // Executables and Libraries
    Executable,
    ShaderSharedLibrary,
    SharedLibrary,

    // CUDA
    Cuda,
    CudaHeader,
    Ptx,
    CuBin,

    // Host Callable
    HostCallable,
    ShaderObjectCode,
    HostHostCallable,

    // Metal
    Metal,
    MetalLib,
    MetalLibAssembly,

    // WebGPU
    Wgsl,
    WgslSpirVAssembly,
    WgslSpirV,

    // Slang VM
    SlangVm,

    // LLVM
    HostObjectCode,
    LlvmHostIr,
    LlvmShaderIr,
}

impl SlangTarget {
    /// Convert to the slangc command-line argument
    pub fn as_arg(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::None => "none",
            Self::Hlsl => "hlsl",
            Self::Dxbc => "dxbc",
            Self::DxbcAssembly => "dxbc-asm",
            Self::Dxil => "dxil",
            Self::DxilAssembly => "dxil-asm",
            Self::Glsl => "glsl",
            Self::SpirV => "spirv",
            Self::SpirVAssembly => "spirv-asm",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CppHeader => "hpp",
            Self::TorchBinding => "torch-binding",
            Self::HostCpp => "host-cpp",
            Self::Executable => "exe",
            Self::ShaderSharedLibrary => "shader-dll",
            Self::SharedLibrary => "dll",
            Self::Cuda => "cuda",
            Self::CudaHeader => "cuh",
            Self::Ptx => "ptx",
            Self::CuBin => "cubin",
            Self::HostCallable => "host-callable",
            Self::ShaderObjectCode => "shader-object-code",
            Self::HostHostCallable => "host-host-callable",
            Self::Metal => "metal",
            Self::MetalLib => "metallib",
            Self::MetalLibAssembly => "metallib-asm",
            Self::Wgsl => "wgsl",
            Self::WgslSpirVAssembly => "wgsl-spirv-asm",
            Self::WgslSpirV => "wgsl-spirv",
            Self::SlangVm => "slang-vm",
            Self::HostObjectCode => "host-object-code",
            Self::LlvmHostIr => "llvm-ir",
            Self::LlvmShaderIr => "llvm-shader-ir",
        }
    }

    /// Parse from a string (supports all aliases)
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "unknown" => Some(Self::Unknown),
            "none" => Some(Self::None),
            "hlsl" => Some(Self::Hlsl),
            "dxbc" => Some(Self::Dxbc),
            "dxbc-asm" | "dxbc-assembly" => Some(Self::DxbcAssembly),
            "dxil" => Some(Self::Dxil),
            "dxil-asm" | "dxil-assembly" => Some(Self::DxilAssembly),
            "glsl" => Some(Self::Glsl),
            "spirv" => Some(Self::SpirV),
            "spirv-asm" | "spirv-assembly" => Some(Self::SpirVAssembly),
            "c" => Some(Self::C),
            "cpp" | "c++" | "cxx" => Some(Self::Cpp),
            "hpp" => Some(Self::CppHeader),
            "torch" | "torch-binding" | "torch-cpp" | "torch-cpp-binding" => Some(Self::TorchBinding),
            "host-cpp" | "host-c++" | "host-cxx" => Some(Self::HostCpp),
            "exe" | "executable" => Some(Self::Executable),
            "shader-sharedlib" | "shader-sharedlibrary" | "shader-dll" => Some(Self::ShaderSharedLibrary),
            "sharedlib" | "sharedlibrary" | "dll" => Some(Self::SharedLibrary),
            "cuda" | "cu" => Some(Self::Cuda),
            "cuh" => Some(Self::CudaHeader),
            "ptx" => Some(Self::Ptx),
            "cuobj" | "cubin" => Some(Self::CuBin),
            "host-callable" | "callable" => Some(Self::HostCallable),
            "object-code" | "shader-object-code" => Some(Self::ShaderObjectCode),
            "host-host-callable" => Some(Self::HostHostCallable),
            "metal" => Some(Self::Metal),
            "metallib" => Some(Self::MetalLib),
            "metallib-asm" => Some(Self::MetalLibAssembly),
            "wgsl" => Some(Self::Wgsl),
            "wgsl-spirv-asm" | "wgsl-spirv-assembly" => Some(Self::WgslSpirVAssembly),
            "wgsl-spirv" => Some(Self::WgslSpirV),
            "slangvm" | "slang-vm" => Some(Self::SlangVm),
            "host-object-code" => Some(Self::HostObjectCode),
            "llvm-host-ir" | "llvm-ir" => Some(Self::LlvmHostIr),
            "llvm-shader-ir" => Some(Self::LlvmShaderIr),
            _ => None,
        }
    }

    /// Check if this target produces binary output
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Self::Dxbc | Self::Dxil | Self::SpirV | Self::Executable
            | Self::ShaderSharedLibrary | Self::SharedLibrary | Self::CuBin
            | Self::MetalLib | Self::WgslSpirV | Self::SlangVm
            | Self::HostObjectCode | Self::ShaderObjectCode
        )
    }

    /// Check if this target is suitable for wgpu
    pub fn is_wgpu_compatible(&self) -> bool {
        matches!(self, Self::SpirV | Self::Wgsl)
    }
}

impl std::fmt::Display for SlangTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_arg())
    }
}

impl std::str::FromStr for SlangTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Unknown Slang target: {}", s))
    }
}

pub enum Profile {
    Sm(String),
    Glsl(String),
    Vs(String),
    Hs(String),
    Ds(String),
    Gs(String),
    Ps(String),
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Profile::Sm(v) => format!("sm_{v}"),
            Profile::Glsl(v) => format!("glsl_{v}"),
            Profile::Vs(v) => format!("vs_{v}"),
            Profile::Hs(v) => format!("hs_{v}"),
            Profile::Ds(v) => format!("ds_{v}"),
            Profile::Gs(v) => format!("gs_{v}"),
            Profile::Ps(v) => format!("ps_{v}"),
        };
        write!(f, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let result = SlangShaderBuilder::new("light")
            .add_source_str(include_str!("../test/light.slang"))
            .entry_with_stage("vs_main", ShaderStage::Vertex)
            .entry_with_stage("fs_main", ShaderStage::Fragment)
            .build(SlangTarget::SpirV);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_bytes() {
        let bytes = vec![1, 2, 3, 4];
        let shader = CompiledSlangShader::from_bytes("idk", &bytes);
        assert_eq!(shader.source, bytes);
        assert_eq!(shader.label(), "unknown");
    }
}