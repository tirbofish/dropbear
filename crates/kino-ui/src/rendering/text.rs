use glam::Vec2;
use glyphon::{
    Buffer, Cache, Color, FontSystem, Resolution, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer, Viewport,
};

pub struct TextEntry {
    pub buffer: Buffer,
    pub position: Vec2,
}

pub struct KinoTextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    pub viewport: Viewport,
    pub entries: Vec<TextEntry>,
}

impl KinoTextRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        log::debug!("Creating KinoTextRenderer");
        log::debug!("Loading system fonts");
        let mut font_system = FontSystem::new();
        font_system.db_mut().load_system_fonts();
        log::debug!("Loaded system fonts");

        log::debug!("Loading \"Roboto-Regular.ttf\" as fallback");
        font_system.db_mut().load_font_data(
            include_bytes!("../../../../resources/fonts/Roboto-Regular.ttf").to_vec(),
        );
        log::debug!("Loaded fallback");
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer = TextRenderer::new(&mut atlas, device, Default::default(), None);

        log::debug!("Created new KinoTextRenderer");

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            viewport,
            entries: Vec::new(),
        }
    }

    /// Prepare the text renderer for drawing
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        self.viewport.update(queue, Resolution { width, height });
        if self.entries.is_empty() {
            return;
        }

        let text_areas: Vec<TextArea> = self
            .entries
            .iter()
            .map(|entry| TextArea {
                buffer: &entry.buffer,
                left: entry.position.x,
                top: entry.position.y,
                bounds: TextBounds {
                    right: width as i32,
                    bottom: height as i32,
                    ..Default::default()
                },
                scale: 1.0,
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            })
            .collect();
        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        self.entries.clear();
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }

    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        self.viewport.update(queue, Resolution { width, height });
    }
}
