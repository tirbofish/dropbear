/// WGPU based traits that are useful for those using the [wgpu] crate.
#[cfg(feature = "use-wgpu")]
pub trait WgpuUtils {
    /// Creates a new [`wgpu::ShaderModuleDescriptor`] for you to create your own shaders with.
    fn create_wgpu_shader(&self) -> wgpu::ShaderModuleDescriptor<'_>;
}

#[cfg(feature = "use-wgpu")]
impl WgpuUtils for crate::CompiledSlangShader {
    fn create_wgpu_shader(&self) -> wgpu::ShaderModuleDescriptor<'_> {
        wgpu::ShaderModuleDescriptor {
            label: Some(self.label.as_str()),
            source: wgpu::util::make_spirv(self.source.as_ref()),
        }
    }
}
