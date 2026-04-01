use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use hecs::{Entity, World};
use glam::{Mat4, Quat, Vec3};
use dropbear_engine::animation::{AnimationComponent, MorphTargetInfo};
use dropbear_engine::asset::{Handle, ASSET_REGISTRY};
use dropbear_engine::billboarding::BillboardPipeline;
use dropbear_engine::buffer::ResizableBuffer;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::graphics::{CommandEncoder, InstanceRaw, SharedGraphicsContext};
use dropbear_engine::lighting::Light;
use dropbear_engine::model::{DrawLight, DrawModel, Material, Mesh, Model};
use dropbear_engine::pipelines::DropbearShaderPipeline;
use dropbear_engine::pipelines::animation::AnimationDefaults;
use dropbear_engine::pipelines::hdr::HdrPipeline;
use dropbear_engine::pipelines::light_cube::LightCubePipeline;
use dropbear_engine::pipelines::shader::MainRenderPipeline;
use dropbear_engine::sky::SkyPipeline;
use kino_ui::KinoState;
use crate::billboard::BillboardComponent;
use crate::debug::DebugDrawExt;
use crate::entity_status::EntityStatus;
use crate::hierarchy::EntityTransformExt;
use crate::physics::collider::ColliderGroup;
use crate::states::SCENES;

pub struct RenderInstance {
    pub entity: Entity,
    pub instance: InstanceRaw,
    pub animation: Option<AnimationBuffers>,
}

pub struct AnimationBuffers {
    pub skinning:      wgpu::Buffer,
    pub morph_weights: wgpu::Buffer,
    pub morph_info:    wgpu::Buffer,
    pub weight_count:  u32,
}

pub struct ModelBatch {
    pub model_id:  u64,
    pub instances: Vec<RenderInstance>,
}

pub struct PreparedModel {
    pub model: Arc<Model>,
    pub handle_id: u64,
    pub instance_count: u32,
    pub entity: Option<Entity>,
}

/// Just common rendering functions that are shared between redback-runtime and eucalyptus-editor.
pub struct RendererCommon;

impl RendererCommon {
    pub fn clear_viewport(graphics: &SharedGraphicsContext, encoder: &mut CommandEncoder, hdr: &HdrPipeline) {
        puffin::profile_scope!("Clearing viewport");
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("viewport clear pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr.render_view(),
                depth_slice: None,
                resolve_target: hdr.resolve_target(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 100.0 / 255.0,
                        g: 149.0 / 255.0,
                        b: 237.0 / 255.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &graphics.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
    }

    pub fn collect_lights(world: &World) -> (Vec<Light>, u32) {
        puffin::profile_scope!("Locating lights");
        let mut lights = Vec::new();
        let mut enabled = 0u32;
        for light in world.query::<&Light>().iter() {
            if light.component.enabled { enabled += 1; }
            lights.push(light.clone());
        }
        (lights, enabled)
    }

    pub fn locate_renderers(
        world: &World,
        batches: &mut HashMap<u64, ModelBatch>,
        graphics: Arc<SharedGraphicsContext>,
        default_skinning_buffer: &Option<wgpu::Buffer>,
    ) {
        puffin::profile_scope!("finding all renderers");

        let mut query = world.query::<(Entity, &MeshRenderer, Option<&mut AnimationComponent>)>();

        for (entity, renderer, animation) in query.iter() {
            if let Ok(status) = world.get::<&EntityStatus>(entity) {
                if status.hidden || status.disabled { continue; }
            }

            let handle = renderer.model();
            if handle.is_null() { continue; }

            let instance_raw = renderer.instance.to_raw();

            let animation_buffers = Self::resolve_animation_buffers(
                graphics.clone(),
                default_skinning_buffer,
                animation
            );

            batches
                .entry(handle.id)
                .or_insert_with(|| ModelBatch { model_id: handle.id, instances: Vec::new() })
                .instances
                .push(RenderInstance {
                    entity,
                    instance: instance_raw,
                    animation: animation_buffers,
                });
        }
    }

    fn resolve_animation_buffers(
        graphics: Arc<SharedGraphicsContext>,
        default_skinning_buffer: &Option<wgpu::Buffer>,
        animation: Option<&mut AnimationComponent>,
    ) -> Option<AnimationBuffers> {
        let anim = animation?;

        let has_skinning = !anim.skinning_matrices.is_empty();
        let has_morph    = !anim.morph_weights.is_empty();

        if !has_skinning && !has_morph {
            return None;
        }

        anim.prepare_gpu_resources(graphics.clone());

        let skinning = anim
            .skinning_buffer
            .as_ref()
            .and_then(|b| Some(b.buffer().clone()))
            .or_else(|| default_skinning_buffer.clone())?;

        let morph_weights = anim.morph_weights_buffer
            .as_ref()
            .map(|b| b.buffer().clone())?;

        let morph_info = anim.morph_info_buffer
            .as_ref()
            .map(|b| b.buffer().clone())?;

        Some(AnimationBuffers {
            skinning,
            morph_weights,
            morph_info,
            weight_count: anim.morph_weight_count,
        })
    }

    pub fn prepare_models(
        graphics: &SharedGraphicsContext,
        batches: &HashMap<u64, ModelBatch>,
        instance_buffer_cache: &mut HashMap<u64, ResizableBuffer<InstanceRaw>>,
    ) -> (Vec<PreparedModel>, HashMap<u64, Arc<Model>>) {
        puffin::profile_scope!("preparing models");
        let registry = ASSET_REGISTRY.read();
        let mut model_cache = HashMap::new();
        let mut prepared = Vec::new();

        for (handle_id, batch) in batches {
            let static_instances: Vec<_> = batch.instances.iter()
                .filter(|i| i.animation.is_none())
                .collect();
            if static_instances.is_empty() { continue; }

            let Some(model) = registry.get_model(Handle::new(*handle_id)) else {
                log_once::error_once!("Missing model handle {} in registry", handle_id);
                continue;
            };

            let instances: Vec<InstanceRaw> = static_instances.iter()
                .map(|i| i.instance)
                .collect();
            let entity = static_instances.first().map(|i| i.entity);

            let instance_buffer = instance_buffer_cache
                .entry(*handle_id)
                .or_insert_with(|| ResizableBuffer::new(
                    &graphics.device,
                    instances.len().max(1),
                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    &format!("resizable buffer<handle={}>", handle_id),
                ));
            instance_buffer.write(&graphics.device, &graphics.queue, &instances);

            model_cache.insert(*handle_id, model.clone());
            prepared.push(PreparedModel {
                model,
                handle_id: *handle_id,
                instance_count: instances.len() as u32,
                entity,
            });
        }

        // also cache models needed for animated instances
        for batch in batches.values() {
            for inst in &batch.instances {
                if inst.animation.is_some() && !model_cache.contains_key(&batch.model_id) {
                    if let Some(model) = registry.get_model(Handle::new(batch.model_id)) {
                        model_cache.insert(batch.model_id, model);
                    }
                }
            }
        }

        (prepared, model_cache)
    }

    pub fn render_light_cubes(
        graphics: &Arc<SharedGraphicsContext>,
        encoder: &mut CommandEncoder,
        hdr: &HdrPipeline,
        lights: &[Light],
        camera: &Camera,

        light_cube_pipeline: Option<&LightCubePipeline>,
    ) {
        let (Some(light_pipeline), Some(l)) = (&light_cube_pipeline, lights.first()) else { return };
        let Some(model) = ASSET_REGISTRY.read().get_model(l.cube_model) else {
            log_once::error_once!("Missing light cube model handle in registry");
            return;
        };

        puffin::profile_scope!("light cube pass");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("light cube render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr.render_view(),
                depth_slice: None,
                resolve_target: hdr.resolve_target(),
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &graphics.depth_texture.view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(light_pipeline.pipeline());
        for light in lights {
            puffin::profile_scope!("rendering light", &light.label);
            pass.set_vertex_buffer(1, light.instance_buffer.buffer().slice(..));
            if !light.component.visible { continue; }
            pass.draw_light_model(&model, &camera.bind_group, &light.bind_group);
        }
    }

    pub fn render_models(
        graphics: &Arc<SharedGraphicsContext>,
        encoder: &mut CommandEncoder,
        hdr: &HdrPipeline,
        world: &World,
        batches: &HashMap<u64, ModelBatch>,
        model_cache: &HashMap<u64, Arc<Model>>,
        per_frame_bind_group: &wgpu::BindGroup,
        environment_bind_group: &wgpu::BindGroup,
        pipeline: &MainRenderPipeline,
        animation_defaults: &AnimationDefaults,
        instance_buffer_cache: &HashMap<u64, ResizableBuffer<InstanceRaw>>,
        animated_instance_buffers: &mut HashMap<Entity, ResizableBuffer<InstanceRaw>>,
        animated_bind_group_cache: &mut HashMap<Entity, (u64, wgpu::BindGroup)>,
        static_bind_group_cache: &mut HashMap<u64, wgpu::BindGroup>,
        last_morph_info_per_mesh: &mut HashMap<u32, MorphTargetInfo>,
    ) {
        puffin::profile_scope!("model render pass");

        for (_, batch) in batches {
            let Some(model) = model_cache.get(&batch.model_id) else { continue };

            let static_count = batch.instances.iter().filter(|i| i.animation.is_none()).count() as u32;
            if static_count > 0 {
                let Some(first) = batch.instances.iter().find(|i| i.animation.is_none()) else { continue };
                let Ok(renderer) = world.get::<&MeshRenderer>(first.entity) else { continue };

                if let Some(deltas) = model.morph_deltas_buffer.as_ref() {
                    if !static_bind_group_cache.contains_key(&batch.model_id) {
                        let bg = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("static model animation bind group"),
                            layout: &graphics.layouts.animation_layout,
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: animation_defaults.skinning_buffer.buffer().as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 1, resource: deltas.as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 2, resource: animation_defaults.morph_weights_buffer.buffer().as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 3, resource: animation_defaults.morph_info_buffer.buffer().as_entire_binding() },
                            ],
                        });
                        static_bind_group_cache.insert(batch.model_id, bg);
                    }
                }

                let animation_bg: &wgpu::BindGroup = if model.morph_deltas_buffer.is_some() {
                    static_bind_group_cache.get(&batch.model_id).unwrap()
                } else {
                    &animation_defaults.animation_bind_group
                };

                let Some(instance_buffer) = instance_buffer_cache.get(&batch.model_id) else { continue };

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("model render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr.render_view(),
                        depth_slice: None,
                        resolve_target: hdr.resolve_target(),
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &graphics.depth_texture.view,
                        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(pipeline.pipeline());
                pass.set_vertex_buffer(1, instance_buffer.slice(static_count as usize));

                for mesh in &model.meshes {
                    let mut weights = mesh.morph_default_weights.clone();
                    let target_count = mesh.morph_target_count as usize;
                    if weights.len() < target_count { weights.resize(target_count, 0.0); }
                    if weights.is_empty() { weights.push(0.0); }

                    graphics.queue.write_buffer(
                        animation_defaults.morph_weights_buffer.buffer(), 0,
                        bytemuck::cast_slice(&weights),
                    );

                    let info = MorphTargetInfo {
                        num_vertices: mesh.morph_vertex_count,
                        num_targets: mesh.morph_target_count,
                        base_offset: mesh.morph_deltas_offset,
                        weight_offset: 0,
                        uses_morph: if mesh.morph_target_count > 0 && !weights.is_empty() { 1 } else { 0 },
                        _padding: Default::default(),
                    };

                    let cache_key = mesh.morph_deltas_offset;
                    let needs_write = last_morph_info_per_mesh.get(&cache_key).map_or(true, |prev| {
                        prev.num_vertices != info.num_vertices
                            || prev.num_targets != info.num_targets
                            || prev.base_offset != info.base_offset
                            || prev.uses_morph != info.uses_morph
                    });
                    if needs_write {
                        graphics.queue.write_buffer(
                            animation_defaults.morph_info_buffer.buffer(), 0,
                            bytemuck::bytes_of(&info),
                        );
                        last_morph_info_per_mesh.insert(cache_key, info);
                    }

                    let material = Self::resolve_material(model, mesh, &renderer);
                    pass.draw_mesh_instanced(mesh, material, 0..static_count, per_frame_bind_group, animation_bg, environment_bind_group);
                }
            }

            for inst in batch.instances.iter().filter(|i| i.animation.is_some()) {
                puffin::profile_scope!("rendering animated model", format!("{:?}", inst.entity));
                let anim = inst.animation.as_ref().unwrap();

                {
                    let buf = animated_instance_buffers.entry(inst.entity).or_insert_with(|| {
                        ResizableBuffer::new(
                            &graphics.device, 1,
                            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            "animated instance buffer",
                        )
                    });
                    buf.write(&graphics.device, &graphics.queue, &[inst.instance]);
                }

                let Ok(renderer) = world.get::<&MeshRenderer>(inst.entity) else { continue };

                let morph_deltas = model.morph_deltas_buffer.as_ref()
                    .map(|b| b as &wgpu::Buffer)
                    .unwrap_or_else(|| animation_defaults.morph_deltas_buffer.buffer());

                let mut hasher = DefaultHasher::new();
                anim.skinning.hash(&mut hasher);
                let stamp = hasher.finish();

                {
                    let cached = animated_bind_group_cache.get(&inst.entity);
                    if cached.map_or(true, |(s, _)| *s != stamp) {
                        let bg = pipeline.animation_bind_group(
                            graphics.clone(),
                            &anim.skinning,
                            morph_deltas,
                            &anim.morph_weights,
                            &anim.morph_info,
                        );
                        animated_bind_group_cache.insert(inst.entity, (stamp, bg));
                    }
                }

                let animation_bg = &animated_bind_group_cache[&inst.entity].1;
                let instance_buffer = &animated_instance_buffers[&inst.entity];

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("animated model render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr.render_view(),
                        depth_slice: None,
                        resolve_target: hdr.resolve_target(),
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &graphics.depth_texture.view,
                        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(pipeline.pipeline());
                pass.set_vertex_buffer(1, instance_buffer.slice(1));

                for mesh in &model.meshes {
                    let mesh_target_count = mesh.morph_target_count.min(anim.weight_count);
                    let info = MorphTargetInfo {
                        num_vertices: mesh.morph_vertex_count,
                        num_targets: mesh_target_count,
                        base_offset: mesh.morph_deltas_offset,
                        weight_offset: 0,
                        uses_morph: if mesh_target_count > 0 { 1 } else { 0 },
                        _padding: Default::default(),
                    };
                    graphics.queue.write_buffer(&anim.morph_info, 0, bytemuck::bytes_of(&info));

                    let material = Self::resolve_material(model, mesh, &renderer);
                    pass.draw_mesh_instanced(mesh, material, 0..1, per_frame_bind_group, animation_bg, environment_bind_group);
                }
            }
        }
    }

    pub fn render_sky(
        graphics: &SharedGraphicsContext,
        encoder: &mut CommandEncoder,
        hdr: &HdrPipeline,
        sky: &SkyPipeline,
    ) {
        puffin::profile_scope!("sky render pass");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sky render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr.render_view(),
                depth_slice: None,
                resolve_target: hdr.resolve_target(),
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &graphics.depth_texture.view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&sky.pipeline);
        pass.set_bind_group(0, &sky.camera_bind_group, &[]);
        pass.set_bind_group(1, &sky.environment_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn render_collider_debug(
        graphics: &SharedGraphicsContext,
        world: &World,
        current_scene_name: Option<&str>,
    ) {
        let show_hitboxes = current_scene_name
            .and_then(|scene_name| {
                let scenes = SCENES.read();
                scenes.iter()
                    .find(|s| s.scene_name == scene_name)
                    .map(|s| s.settings.show_hitboxes)
            })
            .unwrap_or(false);

        if !show_hitboxes { return; }

        puffin::profile_scope!("collider debug draw");
        if let Some(debug_draw) = graphics.debug_draw.lock().as_mut() {
            let colour = [0.0, 1.0, 0.0, 1.0];
            let to_draw: Vec<_> = {
                let mut q = world.query::<(Entity, &ColliderGroup)>();
                q.iter().map(|(e, cg)| (e, cg.colliders.clone())).collect()
            };
            for (entity, colliders) in to_draw {
                let Ok(et) = world.get::<&EntityTransform>(entity) else { continue };
                let world_tf = et.propagate(world, entity);
                drop(et);
                for collider in &colliders {
                    let entity_matrix = world_tf.matrix().as_mat4();
                    let offset_transform = Transform::new().with_offset(collider.translation, collider.rotation);
                    let offset_matrix = offset_transform.matrix().as_mat4();
                    let final_matrix = entity_matrix * offset_matrix;
                    let (scale, rotation, translation) = final_matrix.to_scale_rotation_translation();
                    debug_draw.draw_collider(&collider.shape, translation, scale, rotation, colour);
                }
            }
        }
    }

    pub fn render_billboards(
        graphics: &Arc<SharedGraphicsContext>,
        encoder: &mut CommandEncoder,
        hdr: &HdrPipeline,
        camera: &Camera,
        world: &World,
        kino: Option<&mut KinoState>,
        billboard_pipeline: Option<&BillboardPipeline>,
    ) {
        puffin::profile_scope!("rendering billboard targets");

        let mut kino_views: HashMap<u64, wgpu::TextureView> = HashMap::new();

        if let Some(kino) = kino {
            let mut kino_encoder = CommandEncoder::new(graphics.clone(), Some("kino billboard encoder"));
            kino.render_billboard_targets(&graphics.device, &graphics.queue, &mut kino_encoder);
            if let Err(e) = kino_encoder.submit() {
                log_once::error_once!("Unable to submit billboard kino pass: {}", e);
            }
            kino_views.extend(kino.billboard_render_target_views());
        }

        let Some(billboard_pipeline) = billboard_pipeline else { return };

        let camera_position = camera.position().as_vec3();
        let camera_projection = Mat4::from_cols_array_2d(&camera.uniform.view_proj);

        let single_fallback_view = if kino_views.len() == 1 {
            kino_views.values().next().cloned()
        } else {
            None
        };

        let mut billboards: Vec<(Mat4, wgpu::TextureView)> = Vec::new();
        let mut query = world.query::<(Entity, &BillboardComponent, Option<&EntityTransform>)>();

        for (entity, billboard, entity_transform) in query.iter() {
            puffin::profile_scope!("rendering billboard", format!("{:?}", entity));
            if !billboard.enabled { continue; }

            let entity_id = entity.to_bits().get();
            let texture_view = kino_views.get(&entity_id).cloned()
                .or_else(|| single_fallback_view.clone());
            let Some(texture_view) = texture_view else { continue };

            let position = entity_transform
                .map(|t| t.sync().position.as_vec3())
                .unwrap_or(Vec3::ZERO)
                + billboard.offset;
            let scale = Vec3::new(billboard.world_size.x, billboard.world_size.y, 1.0);

            let rotation = if let Some(r) = billboard.rotation {
                r
            } else {
                let to_camera = (camera_position - position).normalize_or_zero();
                if to_camera.length_squared() > 0.0 {
                    let mut world_up = Vec3::Y;
                    if to_camera.dot(world_up).abs() > 0.999 { world_up = Vec3::X; }
                    let right = world_up.cross(to_camera).normalize_or_zero();
                    let up = to_camera.cross(right).normalize_or_zero();
                    Quat::from_mat3(&glam::Mat3::from_cols(right, up, to_camera))
                } else {
                    Quat::IDENTITY
                }
            };

            billboards.push((Mat4::from_scale_rotation_translation(scale, rotation, position), texture_view));
        }

        if billboards.is_empty() { return; }

        puffin::profile_scope!("billboard render pass");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("billboard render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr.render_view(),
                depth_slice: None,
                resolve_target: hdr.resolve_target(),
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &graphics.depth_texture.view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        for (transform, texture_view) in billboards {
            billboard_pipeline.draw(graphics.clone(), &mut pass, transform, camera_projection, &texture_view);
        }
    }

    fn resolve_material<'a>(model: &'a Model, mesh: &Mesh, renderer: &'a MeshRenderer) -> &'a Material {
        let material = &model.materials[mesh.material];
        renderer.material_snapshot.get(&material.name).unwrap_or_else(|| {
            log_once::warn_once!("Unable to locate MeshRenderer's material_snapshot for that specific material");
            material
        })
    }
}