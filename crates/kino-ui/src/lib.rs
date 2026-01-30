//! kino-ui, a UI library with instruction set UI rendering.
//!
//! Uses wgpu for rendering and winit for input management.

pub mod resp;
pub mod widgets;
pub mod rendering;
pub mod camera;
pub mod asset;
pub mod math;

use crate::asset::{AssetServer, Handle};
use crate::camera::Camera2D;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use crate::rendering::{KinoWGPURenderer, VertexBatch};
use crate::resp::WidgetResponse;
use crate::widgets::{ContaineredWidget, NativeWidget};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use wgpu::{LoadOp, StoreOp};

/// Holds the state of all the instructions, and the vertices+indices for rendering as well
/// as the responses.
pub struct KinoState {
    renderer: KinoWGPURenderer,
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
    pub fn new(renderer: KinoWGPURenderer) -> KinoState {
        log::debug!("Created KinoState");
        KinoState {
            renderer,
            instruction_set: Default::default(),
            widget_states: Default::default(),
            assets: Default::default(),
            batch: Default::default(),
            camera: Camera2D::default(),
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn NativeWidget>) {
        self.instruction_set.push_back(UiInstructionType::Widget(widget));
    }

    pub fn add_container(&mut self, _container: Box<dyn ContaineredWidget>) {
        todo!("This is broken rn and idk how to implement it")
        // self.instruction_set.push_back(UiInstructionType::Containered(
        //     ContaineredWidgetType::Start {
        //         id: container.id(),
        //         widget: container,
        //     }
        // ))
    }

    pub fn add_instruction(&mut self, ui_instruction_type: UiInstructionType) {
        self.instruction_set.push_back(ui_instruction_type);
    }

    /// Polls for changes, builds the tree and prepares them for rendering.
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

    /// Pushes the vertices and indices to the renderer.
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
                log::debug!("Rendering textured geometry: {:?}", tg);
                let texture = tg.texture_id.and_then(|v| {
                    self.assets.get_texture(v)
                });
                self.renderer.draw_batch(&mut pass, device, queue, &mut tg.batch, texture);
            }
        }
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

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct WidgetId(pub u64);

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