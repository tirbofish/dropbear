#![doc = include_str!("../README.md")]

pub mod asset;
pub mod camera;
pub mod math;
pub mod rendering;
pub mod resp;
pub mod widgets;
pub mod windowing;
mod tree;

pub mod crates {
    pub use glyphon;
    pub use wgpu;
    pub use winit;
}

pub use widgets::shorthand::*;
pub use tree::{WidgetDescriptor, WidgetNode, WidgetTree};

use crate::asset::{AssetServer, Handle};
use crate::camera::Camera2D;
use crate::math::Rect;
use crate::rendering::{KinoRenderContext, KinoRenderTargetId, KinoWGPURenderer};
use crate::rendering::text::{KinoTextRenderer, TextEntry};
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use crate::resp::WidgetResponse;
use crate::widgets::{ContaineredWidget, NativeWidget};
use crate::windowing::KinoWinitWindowing;
use glam::Vec2;
use rendering::batching::VertexBatch;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use wgpu::{LoadOp, StoreOp};
use dropbear_utils::StaleTracker;

/// Holds the state of all the instructions, and the vertices+indices for rendering as well
/// as the responses.
pub struct KinoState {
    renderer: KinoWGPURenderer,
    windowing: KinoWinitWindowing,
    current_render_target: Option<KinoRenderTargetId>,
    render_targets: HashMap<KinoRenderTargetId, KinoRenderContext>,
    render_target_cache: StaleTracker<KinoRenderTargetId, KinoRenderContext>,
    batches: HashMap<KinoRenderTargetId, PrimitiveBatch>,
    text_entries: HashMap<KinoRenderTargetId, Vec<TextEntry>>,
    instruction_set: VecDeque<UiInstructionType>,
    widget_states: HashMap<WidgetId, WidgetResponse>,
    assets: AssetServer,
    camera: Camera2D,
    container_stack: Vec<ContainerContext>,
}

// public stuff
impl KinoState {
    /// Returns a mutable reference to the current [`KinoTextRenderer`], used for text and font
    /// management.
    pub fn text(&mut self) -> &mut KinoTextRenderer {
        &mut self.renderer.text
    }

    /// Returns a mutable reference to the current [`KinoWGPURenderer`], used for rendering and
    /// pipelines.
    pub fn renderer(&mut self) -> &mut KinoWGPURenderer {
        &mut self.renderer
    }
    
    /// Returns a mutable reference to the current [`KinoWinitWindowing`], used for handling events
    /// and windowing operations.
    pub fn windowing(&mut self) -> &mut KinoWinitWindowing {
        &mut self.windowing
    }

    /// Returns a mutable reference to the current [`Camera2D`], used for displaying the current
    /// viewport.
    pub fn camera(&mut self) -> &mut Camera2D {
        &mut self.camera
    }

    /// Returns a mutable reference to the [`AssetServer`], used for storing textures and
    /// other assets.
    pub fn assets(&mut self) -> &mut AssetServer {
        &mut self.assets
    }
}

impl KinoState {
    /// Creates a new instance of a [KinoState].
    ///
    /// This sits inside your `init()` function.
    pub fn new(renderer: KinoWGPURenderer, windowing: KinoWinitWindowing) -> Self {
        log::debug!("Created KinoState");
        KinoState {
            renderer,
            windowing,
            current_render_target: None,
            render_targets: Default::default(),
            render_target_cache: Default::default(),
            batches: Default::default(),
            text_entries: Default::default(),
            instruction_set: Default::default(),
            widget_states: Default::default(),
            assets: Default::default(),
            camera: Camera2D::default(),
            container_stack: Default::default(),
        }
    }

    /// Starts the pass for rendering kino ui widgets.
    ///
    /// This function panics if an existing render target has been started without [`self.flush`] not being called
    /// and a new one is created.
    ///
    /// This belongs in your `update()` function at the start of the "render pass".
    pub fn begin(&mut self, render_target: KinoRenderTargetId) {
        if self.current_render_target.is_some() {
            panic!("Tried to begin a new render target while a previous one is still active");
        }

        self.current_render_target = Some(render_target);
    }

    /// This finishes your kino render pass, flushes and builds the widget tree before being polled
    /// for inputs and prepared for GPU upload.
    ///
    /// This function panics if this function is called without an existing render target.
    ///
    /// This belongs in your `update()` function at the end of the "render pass".
    pub fn flush(&mut self) {
        log::trace!("flushing kinostate");
        let Some(render_target) = self.current_render_target else {
            panic!("flush() called without an active render target; call begin() first");
        };

        let current_instructions = { self.instruction_set.drain(..).collect::<Vec<_>>() };

        self.widget_states.clear();

        let tree = Self::build_tree(current_instructions);

        self.render_tree(tree);

        self.render_targets
            .entry(render_target)
            .or_insert(KinoRenderContext::HUD);

        self.current_render_target = None;
    }

    /// Returns a potential [`wgpu::TextureView`] from the render target cache (billboard) based on the provided entity_id.
    pub fn billboard_render_target_view(&mut self, entity_id: u64) -> Option<&wgpu::TextureView> {
        let target = KinoRenderTargetId::Billboard(entity_id);
        match self.render_target_cache.get(&target) {
            Some(KinoRenderContext::Billboard { view, .. }) => Some(view),
            _ => None,
        }
    }

    /// Collects all [`KinoRenderTargetId`] and [`wgpu::TextureView`] pairs from the render target cache.
    pub fn billboard_render_target_views(&self) -> Vec<(u64, wgpu::TextureView)> {
        self.render_target_cache
            .iter()
            .filter_map(|(target, context)| match (target, context) {
                (KinoRenderTargetId::Billboard(entity_id), KinoRenderContext::Billboard { view, .. }) => {
                    Some((*entity_id, view.clone()))
                }
                _ => None,
            })
            .collect()
    }

    pub(crate) fn push_primitive(
        &mut self,
        verts: &[Vertex],
        indices: &[u16],
        texture_id: Option<Handle<Texture>>,
    ) {
        let target = self
            .current_render_target
            .expect("Attempted to push primitives without begin(render_target)");
        self.batches
            .entry(target)
            .or_default()
            .push(verts, indices, texture_id);
    }

    pub(crate) fn push_text_entry(&mut self, entry: TextEntry) {
        let target = self
            .current_render_target
            .expect("Attempted to push text without begin(render_target)");
        self.text_entries.entry(target).or_default().push(entry);
    }

    fn ensure_billboard_target(
        &mut self,
        device: &wgpu::Device,
        target: KinoRenderTargetId,
        width: u32,
        height: u32,
    ) -> wgpu::TextureView {
        let recreate = match self.render_target_cache.get(&target) {
            Some(KinoRenderContext::Billboard { view, .. }) => {
                let tex = view.texture();
                tex.width() != width || tex.height() != height
            }
            _ => true,
        };

        if recreate {
            let (texture, view) = Texture::create_render_target(
                device,
                self.renderer.texture_bind_group_layout(),
                width,
                height,
                self.renderer.format,
            );

            self.render_target_cache
                .insert(target, KinoRenderContext::Billboard { texture, view });
        }

        match self.render_target_cache.get(&target) {
            Some(KinoRenderContext::Billboard { view, .. }) => view.clone(),
            _ => unreachable!("Billboard render target cache entry must be Billboard"),
        }
    }

    fn used_extent(
        geometry: &[TexturedGeometry],
        text_entries: &[TextEntry],
    ) -> (u32, u32) {
        let mut max_extent = Vec2::ZERO;

        for textured in geometry {
            if let Some(max_pos) = textured.batch.max_position() {
                max_extent = max_extent.max(max_pos);
            }
        }

        for entry in text_entries {
            max_extent = max_extent.max(entry.position + entry.size);
        }

        let width = max_extent.x.ceil().max(1.0) as u32;
        let height = max_extent.y.ceil().max(1.0) as u32;
        (width, height)
    }

    /// Renders all billboard targets into their allocated textures.
    pub fn render_billboard_targets(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        #[cfg(feature = "trace")]
        puffin::profile_function!();
        self.render_target_cache.tick();

        let targets = self
            .render_targets
            .keys()
            .copied()
            .filter(|target| {
                matches!(target, KinoRenderTargetId::Billboard(_))
                    && (self.batches.contains_key(target) || self.text_entries.contains_key(target))
            })
            .collect::<Vec<_>>();

        for target in targets {
            #[cfg(feature = "trace")]
            puffin::profile_scope!("rendering target", format!("{:?}", target));
            let mut geometry = self
                .batches
                .remove(&target)
                .map(|mut batch| batch.take())
                .unwrap_or_default();
            let text_entries = self.text_entries.remove(&target).unwrap_or_default();

            if geometry.is_empty() && text_entries.is_empty() {
                continue;
            }

            let (width, height) = Self::used_extent(&geometry, &text_entries);
            let target_view = self.ensure_billboard_target(device, target, width, height);
            self.renderer.upload_camera_matrix(
                queue,
                self.camera
                    .view_proj(Vec2::new(width as f32, height as f32))
                    .to_cols_array_2d(),
            );
            self.renderer.text.entries = text_entries;
            self.renderer.text.prepare(device, queue, width, height);

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("kino billboard render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for mut tg in geometry.drain(..) {
                let texture = tg.texture_id.and_then(|v| self.assets.get_texture(v));
                self.renderer
                    .draw_batch(&mut pass, device, queue, &mut tg.batch, texture);
            }

            self.renderer.text.render(&mut pass);
        }
    }

    /// Adds a widget (a [`NativeWidget`]) to the instruction set as a
    /// [`UiInstructionType::Widget`] and returns back the associated [`WidgetId`] for response
    /// checking.
    pub fn add_widget(&mut self, widget: Box<dyn NativeWidget>) -> WidgetId {
        let id = widget.id();
        self.instruction_set
            .push_back(UiInstructionType::Widget(widget));
        id
    }

    /// Adds a widget (a [`ContaineredWidget`]) to the instruction set as a
    /// [`UiInstructionType::Containered`] and returns the associated [`WidgetId`] for response
    /// checking.
    pub fn add_container(&mut self, container: Box<dyn ContaineredWidget>) -> WidgetId {
        let id = container.id();
        self.instruction_set
            .push_back(UiInstructionType::Containered(
                ContaineredWidgetType::Start {
                    id,
                    widget: container,
                },
            ));
        id
    }

    /// Ends the current container block.
    pub fn end_container(&mut self, id: WidgetId) {
        self.instruction_set
            .push_back(UiInstructionType::Containered(ContaineredWidgetType::End {
                id,
            }));
    }

    /// Adds a [UiInstructionType] to the instruction set.
    pub fn add_instruction(&mut self, ui_instruction_type: UiInstructionType) {
        self.instruction_set.push_back(ui_instruction_type);
    }

    /// Fetches the [`WidgetResponse`] from an associated [`WidgetId`].
    ///
    /// Returns a default [`WidgetResponse`] if not available.
    pub fn response(&self, id: impl Into<WidgetId>) -> WidgetResponse {
        self.widget_states
            .get(&id.into())
            .copied()
            .unwrap_or_default()
    }

    pub fn set_viewport_offset(&mut self, offset: Vec2) {
        self.windowing.viewport_offset = offset;
    }

    /// Sets both the viewport offset (top-left in screen space) and scale (screen->viewport).
    pub fn set_viewport_transform(&mut self, offset: Vec2, scale: Vec2) {
        self.windowing.viewport_offset = offset;
        self.windowing.viewport_scale = scale;
    }

    /// Pushes the vertices and indices to the renderer and renders into an allocated view.
    ///
    /// This is the recommended `render()` function and is used when you want
    /// `kino_ui` to create the render pass and submit to the queue.
    ///
    /// This sits inside your `render()` loop.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        log::trace!("rendering kinostate");
        self.renderer.upload_camera_matrix(
            queue,
            self.camera.view_proj(self.renderer.size).to_cols_array_2d(),
        );
        self.render_target_cache.tick();

        let mut hud_geometry = self
            .batches
            .remove(&KinoRenderTargetId::HUD)
            .map(|mut batch| batch.take())
            .unwrap_or_default();
        self.renderer.text.entries = self
            .text_entries
            .remove(&KinoRenderTargetId::HUD)
            .unwrap_or_default();

        let (width, height) = {
            let tex = view.texture();
            (tex.width(), tex.height())
        };

        self.renderer.text.prepare(device, queue, width, height);

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("kino hud render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for mut tg in hud_geometry.drain(..) {
                let texture = tg.texture_id.and_then(|v| self.assets.get_texture(v));
                self.renderer
                    .draw_batch(&mut pass, device, queue, &mut tg.batch, texture);
            }

            self.renderer.text.render(&mut pass);
        }
    }

    /// Pushes the vertices and indices to the renderer.
    ///
    /// This is used when you want control on the [`wgpu::RenderPass`] and you want
    /// `kino_ui` to only draw the widgets.
    ///
    /// This sits inside your `render()` loop.
    pub fn render_into_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'a>,
    ) {
        self.renderer.upload_camera_matrix(
            queue,
            self.camera.view_proj(self.renderer.size).to_cols_array_2d(),
        );
        let batch = self
            .batches
            .remove(&KinoRenderTargetId::HUD)
            .map(|mut batch| batch.take())
            .unwrap_or_default();
        self.renderer.text.entries = self
            .text_entries
            .remove(&KinoRenderTargetId::HUD)
            .unwrap_or_default();

        for mut tg in batch {
            // log::debug!("Rendering textured geometry: {:?}", tg);
            let texture = tg.texture_id.and_then(|v| self.assets.get_texture(v));
            self.renderer
                .draw_batch(pass, device, queue, &mut tg.batch, texture);
        }

        self.renderer.text.render(pass);
    }

    /// Handles the event into the internal input state.
    ///
    /// This is not required, however if you want reactivity, include it into your WindowEvent code.
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.windowing.handle_event(event);
    }

    /// Creates (or reuses) a texture from raw RGBA bytes or encoded image bytes
    /// (e.g. `include_bytes!()` PNG/JPEG) and stores it by label.
    /// If a texture with the same content hash already exists, it reuses the handle
    /// and just updates the label mapping.
    pub fn add_texture_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: impl Into<String>,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Handle<Texture> {
        let mut raw: Cow<[u8]> = Cow::Borrowed(data);
        let mut width = width;
        let mut height = height;
        let expected_len = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4);

        if data.len() != expected_len {
            match image::load_from_memory(data) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    width = w;
                    height = h;
                    raw = Cow::Owned(rgba.into_raw());
                }
                Err(e) => {
                    log::error!("Failed to decode texture bytes: {}", e);
                    let fallback = Texture::create_default(
                        device,
                        queue,
                        self.renderer.texture_bind_group_layout(),
                        self.renderer.texture_format(),
                    );
                    return self.assets.add_texture_with_label(label, fallback);
                }
            }
        }

        let hash = AssetServer::hash_bytes(raw.as_ref());
        if let Some(handle) = self.assets.texture_handle_by_hash(hash) {
            self.assets.label_texture(label, handle.clone());
            return handle;
        }

        let texture = Texture::from_bytes(
            device,
            queue,
            self.renderer.texture_bind_group_layout(),
            raw.as_ref(),
            width,
            height,
            self.renderer.texture_format(),
        );
        self.assets.add_texture_with_label(label, texture)
    }

    /// Fetch a texture handle by label.
    pub fn texture_handle(&self, label: &str) -> Option<Handle<Texture>> {
        self.assets.get_texture_handle(label)
    }

    /// Returns a reference to the windowing. Used for input detection.
    pub(crate) fn input(&self) -> &KinoWinitWindowing {
        &self.windowing
    }

    pub(crate) fn set_response(&mut self, id: WidgetId, response: WidgetResponse) {
        self.widget_states.insert(id, response);
    }

    pub(crate) fn layout_offset(&self) -> Vec2 {
        self.container_stack
            .last()
            .map(|ctx| ctx.offset)
            .unwrap_or(Vec2::ZERO)
    }

    pub(crate) fn clip_contains(&self, point: Vec2) -> bool {
        match self.container_stack.last().and_then(|ctx| ctx.clip) {
            Some(rect) => rect.contains(point),
            None => self.container_stack.is_empty(),
        }
    }

    pub(crate) fn push_container(&mut self, rect: Rect) {
        let parent_offset = self.layout_offset();
        let world_rect = Rect::new(rect.position + parent_offset, rect.size);
        let clip = match self.container_stack.last().and_then(|ctx| ctx.clip) {
            Some(parent_clip) => intersect_rects(parent_clip, world_rect),
            None => Some(world_rect),
        };
        self.container_stack.push(ContainerContext {
            offset: world_rect.position,
            clip,
        });
    }

    pub(crate) fn pop_container(&mut self) {
        self.container_stack.pop();
    }

    fn build_tree(instructions: Vec<UiInstructionType>) -> Vec<UiNode> {
        let mut stack: Vec<UiNode> = Vec::new();
        let mut root = Vec::new();

        for instruction in instructions {
            match &instruction {
                UiInstructionType::Containered(container_ty) => match container_ty {
                    ContaineredWidgetType::Start { .. } => {
                        stack.push(UiNode {
                            instruction,
                            children: Vec::new(),
                        });
                    }
                    ContaineredWidgetType::End { .. } => {
                        if let Some(node) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(node);
                            } else {
                                root.push(node);
                            }
                        }
                    }
                },
                UiInstructionType::Widget(_) => {
                    let node = UiNode {
                        instruction,
                        children: Vec::new(),
                    };

                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        root.push(node);
                    }
                }
            }
        }

        root
    }

    pub(crate) fn render_tree(&mut self, nodes: Vec<UiNode>) {
        for node in nodes {
            match node.instruction {
                UiInstructionType::Containered(container_ty) => {
                    match container_ty {
                        ContaineredWidgetType::Start { widget, .. } => {
                            log::trace!("Rendering containered widget START");
                            widget.render(node.children, self);
                        }
                        ContaineredWidgetType::End { .. } => {
                            log::trace!("Rendering end widget END");
                            // already handled in tree building
                        }
                    }
                }
                UiInstructionType::Widget(widget) => {
                    log::trace!("Rendering widget: {:?}", widget.id());
                    widget.render(self);
                }
            }
        }
    }
}

/// The id of the widget, often being a hash.
#[cfg_attr(any(feature = "ser"), derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct WidgetId(u64);

impl Default for WidgetId {
    fn default() -> Self {
        WidgetId(0) // dummy value
    }
}

impl WidgetId {
    /// Creates a new [`WidgetId`] from an object that can be hashed.
    pub fn new<H: Hash>(value: H) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        WidgetId(hasher.finish())
    }

    /// Creates a new [`WidgetId`] from a simple u64 value.
    pub fn from_raw(value: u64) -> Self {
        WidgetId(value)
    }

    pub fn get_id(&self) -> u64 {
        self.0
    }
}

impl Into<WidgetId> for &str {
    fn into(self) -> WidgetId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        WidgetId(hasher.finish())
    }
}

impl Into<WidgetId> for String {
    fn into(self) -> WidgetId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        WidgetId(hasher.finish())
    }
}

impl Into<WidgetId> for u64 {
    fn into(self) -> WidgetId {
        WidgetId(self)
    }
}

impl<T: Hash, U: Hash> Into<WidgetId> for (T, U) {
    fn into(self) -> WidgetId {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        self.1.hash(&mut hasher);
        WidgetId(hasher.finish())
    }
}

pub enum UiInstructionType {
    Containered(ContaineredWidgetType),
    Widget(Box<dyn NativeWidget>),
}

pub enum ContaineredWidgetType {
    Start {
        id: WidgetId,
        widget: Box<dyn ContaineredWidget>,
    },
    End {
        id: WidgetId,
    },
}

pub struct UiNode {
    pub instruction: UiInstructionType,
    pub children: Vec<UiNode>,
}

#[derive(Clone, Copy, Debug)]
struct ContainerContext {
    offset: Vec2,
    clip: Option<Rect>,
}

#[derive(Debug, Default)]
pub struct TexturedGeometry {
    pub texture_id: Option<Handle<Texture>>,
    pub batch: VertexBatch,
}

#[derive(Default)]
pub struct PrimitiveBatch{
    geometry: Vec<TexturedGeometry>,
}

impl PrimitiveBatch {
    /// Add verts & indices to batch, preserving submission order & batching consecutive geometry per texture
    pub fn push(&mut self, verts: &[Vertex], indices: &[u16], texture_id: Option<Handle<Texture>>) {
        if let Some(tg) = self.geometry.last_mut()
            && tg.texture_id == texture_id
        {
            tg.batch.push(verts, indices);
            return;
        }

        let mut batch = VertexBatch::default();
        batch.push(verts, indices);
        self.geometry.push(TexturedGeometry { texture_id, batch });
    }

    pub(crate) fn take(&mut self) -> Vec<TexturedGeometry> {
        std::mem::take(&mut self.geometry)
    }
}

fn intersect_rects(a: Rect, b: Rect) -> Option<Rect> {
    let min = a.min().max(b.min());
    let max = a.max().min(b.max());
    if max.cmpge(min).all() {
        Some(Rect::new(min, max - min))
    } else {
        None
    }
}
