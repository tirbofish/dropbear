#![doc = include_str!("../README.md")]

pub mod resp;
pub mod widgets;
pub mod rendering;
pub mod camera;
pub mod asset;
pub mod math;
pub mod windowing;

use crate::asset::{AssetServer, Handle};
use crate::camera::Camera2D;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use crate::rendering::{KinoWGPURenderer};
use crate::resp::WidgetResponse;
use crate::widgets::{ContaineredWidget, NativeWidget};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use wgpu::{LoadOp, StoreOp};
use rendering::batching::VertexBatch;
use crate::windowing::KinoWinitWindowing;
use glam::Vec2;

/// Holds the state of all the instructions, and the vertices+indices for rendering as well
/// as the responses.
pub struct KinoState {
    renderer: KinoWGPURenderer,
    windowing: KinoWinitWindowing,
    instruction_set: VecDeque<UiInstructionType>,
    widget_states: HashMap<WidgetId, WidgetResponse>,
    assets: AssetServer,
    batch: PrimitiveBatch,
    camera: Camera2D,
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
            instruction_set: Default::default(),
            widget_states: Default::default(),
            assets: Default::default(),
            batch: Default::default(),
            camera: Camera2D::default(),
        }
    }

    /// Adds a widget (a [`NativeWidget`]) to the instruction set as a
    /// [`UiInstructionType::Widget`] and returns back the associated [`WidgetId`] for response
    /// checking.
    pub fn add_widget(&mut self, widget: Box<dyn NativeWidget>) -> WidgetId {
        let id = widget.id();
        self.instruction_set.push_back(UiInstructionType::Widget(widget));
        id
    }

    /// Adds a widget (a [`ContaineredWidget`]) to the instruction set as a
    /// [`UiInstructionType::Containered`] and returns the associated [`WidgetId`] for response
    /// checking.
    pub fn add_container(&mut self, _container: Box<dyn ContaineredWidget>) -> WidgetId {
        todo!("This is broken rn and idk how to implement it")
        // self.instruction_set.push_back(UiInstructionType::Containered(
        //     ContaineredWidgetType::Start {
        //         id: container.id(),
        //         widget: container,
        //     }
        // ))
    }

    /// Adds a [UiInstructionType] to the instruction set.
    pub fn add_instruction(&mut self, ui_instruction_type: UiInstructionType) {
        self.instruction_set.push_back(ui_instruction_type);
    }

    /// Polls for changes by clearing the current instruction set, build the tree and
    /// preparing them for rendering.
    ///
    /// If you create a widget and then check for a response before polling, you will not receive
    /// back a response. You are required to poll/prepare the contents before being given access
    /// to the response information.
    ///
    /// This sits inside your `update()` loop.
    pub fn poll(&mut self) {
        log::trace!("polling kinostate");
        let current_instructions = {
            self.instruction_set.drain(..).collect::<Vec<_>>()
        };

        self.widget_states.clear();

        let tree = Self::build_tree(current_instructions);

        self.render_tree(tree);
    }

    /// Fetches the [`WidgetResponse`] from an associated [`WidgetId`].
    ///
    /// This will only provide the proper information **after** you have
    /// polled with [`KinoState::poll`].
    pub fn response(&self, id: WidgetId) -> WidgetResponse {
        self.widget_states.get(&id).copied().unwrap_or_default()
    }

    pub fn set_viewport_offset(&mut self, offset: Vec2) {
        self.windowing.viewport_offset = offset;
    }

    /// Pushes the vertices and indices to the renderer.
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
            self.camera
                .view_proj(self.renderer.size)
                .to_cols_array_2d(),
        );
        let batch = self.batch.take();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("kino render pass"),
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

            for mut tg in batch {
                // log::debug!("Rendering textured geometry: {:?}", tg);
                let texture = tg.texture_id.and_then(|v| {
                    self.assets.get_texture(v)
                });
                self.renderer.draw_batch(&mut pass, device, queue, &mut tg.batch, texture);
            }

            // self.renderer.text.render(&mut pass);
        }
    }

    /// Pushes the vertices and indices to the renderer.
    ///
    /// This is used when you want control on the [`wgpu::RenderPass`] and you want
    /// `kino_ui` to only draw the widgets.
    ///
    /// This sits inside your `render()` loop.
    pub fn render_into_pass(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.renderer.upload_camera_matrix(
            queue,
            self.camera
                .view_proj(self.renderer.size)
                .to_cols_array_2d(),
        );
        let batch = self.batch.take();

        for mut tg in batch {
            // log::debug!("Rendering textured geometry: {:?}", tg);
            let texture = tg.texture_id.and_then(|v| {
                self.assets.get_texture(v)
            });
            self.renderer.draw_batch(pass, device, queue, &mut tg.batch, texture);
        }

        // self.renderer.text.render(&mut pass);
    }

    /// Handles the event into the internal input state.
    ///
    /// This is not required, however if you want reactivity, include it into your WindowEvent code.
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.windowing.handle_event(event);
    }

    /// Returns a mutable reference to the internal [`AssetServer`], used
    /// for storing textures.
    pub fn assets(&mut self) -> &mut AssetServer {
        &mut self.assets
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
        );
        self.assets.add_texture_with_label(label, texture)
    }

    /// Fetch a texture handle by label.
    pub fn texture_handle(&self, label: &str) -> Option<Handle<Texture>> {
        self.assets.get_texture_handle(label)
    }

    pub(crate) fn input(&self) -> &KinoWinitWindowing {
        &self.windowing
    }

    pub(crate) fn set_response(&mut self, id: WidgetId, response: WidgetResponse) {
        self.widget_states.insert(id, response);
    }

    fn build_tree(instructions: Vec<UiInstructionType>) -> Vec<UiNode> {
        let mut stack: Vec<UiNode> = Vec::new();
        let mut root = Vec::new();

        for instruction in instructions {
            match &instruction {
                UiInstructionType::Containered(container_ty) => {
                    match container_ty {
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
                    }
                }
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

    fn render_tree(&mut self, nodes: Vec<UiNode>) {
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
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub fn from_str(str: &str) -> Self {
        str.into()
    }
}

impl Into<WidgetId> for &str {
    fn into(self) -> WidgetId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
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
    }
}

pub struct UiNode {
    pub instruction: UiInstructionType,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Default)]
pub struct TexturedGeometry {
    pub texture_id: Option<Handle<Texture>>,
    pub batch: VertexBatch,
}

#[derive(Default)]
pub struct PrimitiveBatch {
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