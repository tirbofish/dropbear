use std::sync::Arc;
use glam::{Mat4, Quat, Vec3, Vec4};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages, CompareFunction, DepthStencilState, LoadOp, MultisampleState, Operations, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp, TextureFormat, VertexBufferLayout, VertexState};
use crate::buffer::{ResizableBuffer, UniformBuffer};
use crate::graphics::{CommandEncoder, SharedGraphicsContext};
use crate::shader::Shader;

pub struct DebugDraw {
    pipeline: Arc<DebugDrawPipeline>,
    vertices: Vec<DebugVertex>,
    vertex_buffer: ResizableBuffer<DebugVertex>,
}

// main parts
impl DebugDraw {
    /// Creates a new [`DebugDraw`] instance, setting up all `wgpu` related structs such as pipelines
    /// and buffers.
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let pipeline = Arc::new(DebugDrawPipeline::new(graphics.clone()));
        let vertices = vec![];
        let vertex_buffer = ResizableBuffer::new(
            &graphics.device,
            1024,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            "debug draw vertex buffer"
        );

        Self {
            pipeline,
            vertices,
            vertex_buffer,
        }
    }

    /// Flushes away all of the vertices to be drawn, and renders them at that instant.
    pub fn flush(&mut self, graphics: Arc<SharedGraphicsContext>, encoder: &mut CommandEncoder, view_proj: Mat4) {
        if self.vertices.is_empty() {
            return;
        }

        self.vertex_buffer.write(
            &graphics.device,
            &graphics.queue,
            bytemuck::cast_slice(&self.vertices)
        );

        self.pipeline.draw(
            graphics,
            encoder,
            view_proj,
            &self.vertex_buffer,
            self.vertices.len() as u32,
        );

        self.vertices.clear();
    }
}

// helpers
impl DebugDraw {
    // primitives
    /// Draws a line between 2 points with a specific colour. Probably the most primitive of them all.
    pub fn draw_line(&mut self, a: Vec3, b: Vec3, colour: [f32; 4]) {
        let a = a.to_array();
        let b = b.to_array();
        self.vertices.push(DebugVertex { position: [a[0], a[1], a[2], 0.0], colour });
        self.vertices.push(DebugVertex { position: [b[0], b[1], b[2], 0.0], colour });
    }

    /// A wrapper for [draw_line](Self::draw_line), which draws a line from an origin and at a direction
    /// with a specific colour. 
    pub fn draw_ray(&mut self, origin: Vec3, dir: Vec3, colour: [f32; 4]) {
        self.draw_line(origin, origin + dir, colour);
    }

    /// Draws a line from `a` to `b` with a specific colour and an arrowhead at the end.
    pub fn draw_arrow(&mut self, a: Vec3, b: Vec3, colour: [f32; 4]) {
        self.draw_line(a, b, colour);

        // arrowhead — two short lines branching back from the tip
        let dir = (b - a).normalize();
        let len = (b - a).length() * 0.15;

        // find a perpendicular axis
        let up = if dir.dot(Vec3::Y).abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let right = dir.cross(up).normalize();

        let tip = b;
        let base = tip - dir * len;
        self.draw_line(tip, base + right * len * 0.5, colour);
        self.draw_line(tip, base - right * len * 0.5, colour);
    }

    /// Draws a cross/asterisk at `pos` with the given `size`.
    pub fn draw_point(&mut self, pos: Vec3, size: f32, colour: [f32; 4]) {
        let h = size * 0.5;
        self.draw_line(pos - Vec3::X * h, pos + Vec3::X * h, colour);
        self.draw_line(pos - Vec3::Y * h, pos + Vec3::Y * h, colour);
        self.draw_line(pos - Vec3::Z * h, pos + Vec3::Z * h, colour);
    }

    // shapes
    /// Draws a circle in 3D space at `center` with the given `radius`.
    ///
    /// `normal` defines the axis the circle faces. e.g. `Vec3::Y` for a flat ground circle.
    pub fn draw_circle(&mut self, center: Vec3, radius: f32, normal: Vec3, colour: [f32; 4]) {
        let segments = 32;

        // build tangent frame from normal
        let up = if normal.dot(Vec3::Y).abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let tangent = normal.cross(up).normalize();
        let bitangent = normal.cross(tangent).normalize();

        let mut prev = center + tangent * radius;
        for i in 1..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let next = center + (tangent * angle.cos() + bitangent * angle.sin()) * radius;
            self.draw_line(prev, next, colour);
            prev = next;
        }
    }

    /// Draws 3 circles at [`Vec3::X`], [`Vec3::Y`], and [`Vec3::Z`] to make an imitation of a sphere.
    ///
    /// To see a proper sphere, use [`draw_globe`](Self::draw_globe).
    pub fn draw_sphere(&mut self, center: Vec3, radius: f32, colour: [f32; 4]) {
        self.draw_circle(center, radius, Vec3::X, colour);
        self.draw_circle(center, radius, Vec3::Y, colour);
        self.draw_circle(center, radius, Vec3::Z, colour);
    }

    /// Draws a wireframe sphere using latitude and longitude lines, giving a globe-like appearance.
    ///
    /// `lat_lines` controls the number of horizontal rings (latitude),
    /// `lon_lines` controls the number of vertical rings (longitude).
    pub fn draw_globe(&mut self, center: Vec3, radius: f32, lat_lines: u32, lon_lines: u32, color: [f32; 4]) {
        // latitude rings (horizontal circles stacked along Y axis)
        for i in 1..lat_lines {
            let angle = std::f32::consts::PI * (i as f32 / lat_lines as f32); // 0..PI
            let y = angle.cos() * radius;
            let ring_radius = angle.sin() * radius;
            self.draw_circle(center + Vec3::Y * y, ring_radius, Vec3::Y, color);
        }

        // longitude rings (vertical circles rotating around Y axis)
        for i in 0..lon_lines {
            let angle = std::f32::consts::TAU * (i as f32 / lon_lines as f32); // 0..TAU
            let normal = Vec3::new(angle.cos(), 0.0, angle.sin());
            self.draw_circle(center, radius, normal, color);
        }
    }

    /// Draws a wireframe axis-aligned bounding box (AABB) from `min` to `max`.
    ///
    /// Also used for rendering a cube at a minimum position and a maximum position.
    pub fn draw_aabb(&mut self, min: Vec3, max: Vec3, colour: [f32; 4]) {
        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ];

        // bottom face
        self.draw_line(corners[0], corners[1], colour);
        self.draw_line(corners[1], corners[2], colour);
        self.draw_line(corners[2], corners[3], colour);
        self.draw_line(corners[3], corners[0], colour);
        // top face
        self.draw_line(corners[4], corners[5], colour);
        self.draw_line(corners[5], corners[6], colour);
        self.draw_line(corners[6], corners[7], colour);
        self.draw_line(corners[7], corners[4], colour);
        // verticals
        self.draw_line(corners[0], corners[4], colour);
        self.draw_line(corners[1], corners[5], colour);
        self.draw_line(corners[2], corners[6], colour);
        self.draw_line(corners[3], corners[7], colour);
    }

    /// Draws a wireframe oriented bounding box (OBB) at `center`.
    ///
    /// `half_extents` defines the box dimensions along each local axis.
    /// `rotation` orients the box in world space.
    pub fn draw_obb(&mut self, center: Vec3, half_extents: Vec3, rotation: Quat, colour: [f32; 4]) {
        // rotate the 8 unit corners then scale by half_extents
        let corners_local = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new( 1.0, -1.0, -1.0),
            Vec3::new( 1.0,  1.0, -1.0),
            Vec3::new(-1.0,  1.0, -1.0),
            Vec3::new(-1.0, -1.0,  1.0),
            Vec3::new( 1.0, -1.0,  1.0),
            Vec3::new( 1.0,  1.0,  1.0),
            Vec3::new(-1.0,  1.0,  1.0),
        ];

        let corners: Vec<Vec3> = corners_local
            .iter()
            .map(|&c| center + rotation * (c * half_extents))
            .collect();

        // same edge layout as AABB
        self.draw_line(corners[0], corners[1], colour);
        self.draw_line(corners[1], corners[2], colour);
        self.draw_line(corners[2], corners[3], colour);
        self.draw_line(corners[3], corners[0], colour);
        self.draw_line(corners[4], corners[5], colour);
        self.draw_line(corners[5], corners[6], colour);
        self.draw_line(corners[6], corners[7], colour);
        self.draw_line(corners[7], corners[4], colour);
        self.draw_line(corners[0], corners[4], colour);
        self.draw_line(corners[1], corners[5], colour);
        self.draw_line(corners[2], corners[6], colour);
        self.draw_line(corners[3], corners[7], colour);
    }

    /// Draws a wireframe capsule between points `a` (bottom) and `b` (top) with the given `radius`.
    ///
    /// Rendered as two end circles, four connecting lines, and hemispherical arcs on each cap.
    pub fn draw_capsule(&mut self, a: Vec3, b: Vec3, radius: f32, colour: [f32; 4]) {
        let axis = (b - a).normalize();

        // two end circles
        self.draw_circle(a, radius, axis, colour);
        self.draw_circle(b, radius, axis, colour);

        // 4 connecting lines along the sides
        let up = if axis.dot(Vec3::Y).abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let tangent = axis.cross(up).normalize();
        let bitangent = axis.cross(tangent).normalize();

        for dir in [tangent, -tangent, bitangent, -bitangent] {
            self.draw_line(a + dir * radius, b + dir * radius, colour);
        }
    }

    /// Draws a wireframe cylinder centered at `center`, aligned to `axis`.
    pub fn draw_cylinder(&mut self, center: Vec3, half_height: f32, radius: f32, axis: Vec3, colour: [f32; 4]) {
        let axis = axis.normalize();
        let top = center + axis * half_height;
        let bottom = center - axis * half_height;

        self.draw_circle(top, radius, axis, colour);
        self.draw_circle(bottom, radius, axis, colour);

        let up = if axis.dot(Vec3::Y).abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let tangent = axis.cross(up).normalize();
        let bitangent = axis.cross(tangent).normalize();

        for dir in [tangent, -tangent, bitangent, -bitangent] {
            self.draw_line(top + dir * radius, bottom + dir * radius, colour);
        }
    }

    /// Draws a wireframe cone from `apex` extending in `dir`.
    ///
    /// `angle` is the half-angle of the cone in radians.
    ///
    /// `length` controls how far the cone extends from the apex.
    pub fn draw_cone(&mut self, apex: Vec3, dir: Vec3, angle: f32, length: f32, colour: [f32; 4]) {
        let base_center = apex + dir.normalize() * length;
        let base_radius = length * angle.tan();

        // base circle
        self.draw_circle(base_center, base_radius, dir, colour);

        // 4 lines from apex to base rim
        let up = if dir.normalize().dot(Vec3::Y).abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let tangent = dir.cross(up).normalize();
        let bitangent = dir.cross(tangent).normalize();

        for side in [tangent, -tangent, bitangent, -bitangent] {
            self.draw_line(apex, base_center + side * base_radius, colour);
        }
    }

    /// Draws a wireframe frustum by unprojecting the 8 NDC corners using the inverse of `view_proj`.
    ///
    /// Pass in `camera.view_proj()` to visualise the camera's view frustum.
    pub fn draw_frustum(&mut self, view_proj: Mat4, colour: [f32; 4]) {
        // NDC corners, unproject back to world space
        let ndc_corners = [
            Vec3::new(-1.0, -1.0, 0.0), // near
            Vec3::new( 1.0, -1.0, 0.0),
            Vec3::new( 1.0,  1.0, 0.0),
            Vec3::new(-1.0,  1.0, 0.0),
            Vec3::new(-1.0, -1.0, 1.0), // far
            Vec3::new( 1.0, -1.0, 1.0),
            Vec3::new( 1.0,  1.0, 1.0),
            Vec3::new(-1.0,  1.0, 1.0),
        ];

        let inv = view_proj.inverse();

        let corners: Vec<Vec3> = ndc_corners.iter().map(|&ndc| {
            let clip = Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);
            let world = inv * clip;
            world.truncate() / world.w
        }).collect();

        // near face
        self.draw_line(corners[0], corners[1], colour);
        self.draw_line(corners[1], corners[2], colour);
        self.draw_line(corners[2], corners[3], colour);
        self.draw_line(corners[3], corners[0], colour);
        // far face
        self.draw_line(corners[4], corners[5], colour);
        self.draw_line(corners[5], corners[6], colour);
        self.draw_line(corners[6], corners[7], colour);
        self.draw_line(corners[7], corners[4], colour);
        // connecting edges
        self.draw_line(corners[0], corners[4], colour);
        self.draw_line(corners[1], corners[5], colour);
        self.draw_line(corners[2], corners[6], colour);
        self.draw_line(corners[3], corners[7], colour);
    }

    // curves and paths

    /// Draws a polyline through a slice of points, connecting each adjacent pair with a line.
    pub fn draw_polyline(&mut self, points: &[Vec3], colour: [f32; 4]) {
        for window in points.windows(2) {
            self.draw_line(window[0], window[1], colour);
        }
    }

    /// Draws a Catmull-Rom spline through `points`, tessellated into `segments_per_span` line segments per span.
    /// Requires at least 4 control points. The curve passes through all points except the first and last,
    /// which act as phantom tangent guides.
    ///
    /// Duplicate the first and last point if you want the curve to reach the endpoints.
    pub fn draw_spline(&mut self, points: &[Vec3], segments_per_span: u32, colour: [f32; 4]) {
        if points.len() < 4 {
            return;
        }

        for i in 0..points.len().saturating_sub(3) {
            let (p0, p1, p2, p3) = (points[i], points[i+1], points[i+2], points[i+3]);
            let mut prev = p1; // catmull-rom passes through p1..p_{n-2}
            for j in 1..=segments_per_span {
                let t = j as f32 / segments_per_span as f32;
                let next = catmull_rom(p0, p1, p2, p3, t);
                self.draw_line(prev, next, colour);
                prev = next;
            }
        }
    }

    /// Draws a point cross at each vertex position in `vertices`.
    pub fn draw_vertices(&mut self, vertices: &[Vec3], colour: [f32; 4]) {
        for &v in vertices {
            self.draw_point(v, 0.05, colour);
        }
    }
}

fn catmull_rom(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * (
        (2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
    )
}

pub struct DebugDrawPipeline {
    uniform: UniformBuffer<Mat4>,
    bind_group: wgpu::BindGroup,
    pipeline: RenderPipeline,
}

impl DebugDrawPipeline {
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let shader = Shader::new(graphics.clone(), include_str!("shaders/basic.wgsl"), Some("basic shader module"));

        let bind_group_layout = graphics.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("basic camera bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let camera_uniform: UniformBuffer<Mat4> = UniformBuffer::new(&graphics.device, "basic camera uniform");

        let bind_group = graphics.device.create_bind_group(&BindGroupDescriptor {
            label: Some("basic camera bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(camera_uniform.buffer().as_entire_buffer_binding()),
                }
            ],
        });

        let pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("debug draw pipeline layout"),
            bind_group_layouts: &[
                &bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let hdr_format = graphics.hdr.read().format();
        let sample_count: u32 = (*graphics.antialiasing.read()).into();

        let pipeline = graphics.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("debug draw render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[DebugVertex::LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: hdr_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: Default::default(),
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,  // don't write to depth, just read
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform: camera_uniform,
            bind_group,
        }
    }

    pub fn draw(
        &self,
        graphics: Arc<SharedGraphicsContext>,
        encoder: &mut CommandEncoder,
        view_proj: Mat4,
        vertex_buffer: &ResizableBuffer<DebugVertex>,
        vertex_count: u32,
    ) {
        // update camera uniform
        self.uniform.write(&graphics.queue, &view_proj);

        if vertex_count == 0 {
            return;
        }

        let hdr = graphics.hdr.read();

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("debug draw pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: hdr.view(),
                depth_slice: None,
                resolve_target: hdr.resolve_target(),
                ops: Operations {
                    load: LoadOp::Load,   // draw on top of existing frame
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &graphics.depth_texture.view,
                depth_ops: Some(Operations {
                    load: LoadOp::Load,   // read existing depth
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.buffer().slice(..));
        pass.draw(0..vertex_count, 0..1);
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DebugVertex {
    pub position: [f32; 4], // ignore w value, used to ensure 16 bit pads
    pub colour: [f32; 4],
}

impl DebugVertex {
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
        ],
    };
}