use crate::buffer::{ResizableBuffer, UniformBuffer};
use crate::graphics::SharedGraphicsContext;
use crate::model::{AnimationInterpolation, ChannelValues, Model, NodeTransform};
use glam::Mat4;
use std::collections::HashMap;
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MorphTargetInfo {
    pub num_vertices: u32,
    pub num_targets: u32,
    pub base_offset: u32,
    pub weight_offset: u32,
    pub uses_morph: u32, // 0,1 bool
    pub _padding: [u32; 3],
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AnimationComponent {
    #[serde(default)]
    pub active_animation_index: Option<usize>,
    #[serde(default)]
    pub time: f32,
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub is_playing: bool,

    #[serde(default)]
    pub animation_settings: HashMap<usize, AnimationSettings>,

    #[serde(skip)]
    pub local_pose: HashMap<usize, NodeTransform>,
    #[serde(skip)]
    pub skinning_matrices: Vec<Mat4>,

    #[serde(skip)]
    pub skinning_buffer: Option<ResizableBuffer<Mat4>>,
    #[serde(skip)]
    pub morph_deltas_buffer: Option<ResizableBuffer<f32>>,
    #[serde(skip)]
    pub morph_weights_buffer: Option<ResizableBuffer<f32>>,
    #[serde(skip)]
    pub morph_info_buffer: Option<UniformBuffer<MorphTargetInfo>>,

    #[serde(skip)]
    pub available_animations: Vec<String>,

    #[serde(skip)]
    pub last_animation_index: Option<usize>,

    #[serde(skip)]
    pub morph_weights: HashMap<usize, Vec<f32>>,

    #[serde(skip)]
    pub morph_weight_count: u32,
}

impl Clone for AnimationComponent {
    fn clone(&self) -> Self {
        Self {
            active_animation_index: self.active_animation_index,
            time: self.time,
            speed: self.speed,
            looping: self.looping,
            is_playing: self.is_playing,
            animation_settings: self.animation_settings.clone(),
            local_pose: HashMap::new(),
            skinning_matrices: Vec::new(),
            skinning_buffer: None,
            morph_deltas_buffer: None,
            morph_weights_buffer: None,
            morph_info_buffer: None,
            available_animations: Vec::new(),
            last_animation_index: None,
            morph_weights: HashMap::new(),
            morph_weight_count: 0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimationSettings {
    #[serde(default)]
    pub time: f32,
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub is_playing: bool,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
            looping: true,
            is_playing: true,
        }
    }
}

impl Default for AnimationComponent {
    fn default() -> Self {
        Self {
            active_animation_index: None,
            time: 0.0,
            speed: 1.0,
            looping: true,
            is_playing: true,
            animation_settings: HashMap::new(),
            local_pose: HashMap::new(),
            skinning_matrices: Vec::new(),
            available_animations: vec![],
            last_animation_index: None,
            morph_weights: HashMap::new(),
            morph_weight_count: 0,
            skinning_buffer: None,
            morph_deltas_buffer: None,
            morph_weights_buffer: None,
            morph_info_buffer: None,
        }
    }
}

impl AnimationComponent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dt: f32, model: &Model) {
        puffin::profile_function!(&model.label);
        self.available_animations = model
            .animations
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<_>>();

        if self.active_animation_index != self.last_animation_index {
            self.local_pose.clear();
            self.morph_weights.clear();
            self.last_animation_index = self.active_animation_index;
        }

        let Some(anim_idx) = self.active_animation_index else {
            self.reset_to_bind_pose(model);
            return;
        };

        if anim_idx >= model.animations.len() {
            self.reset_to_bind_pose(model);
            return;
        }

        let settings =
            self.animation_settings
                .entry(anim_idx)
                .or_insert_with(|| AnimationSettings {
                    time: self.time,
                    speed: self.speed,
                    looping: self.looping,
                    is_playing: self.is_playing,
                });

        let animation = &model.animations[anim_idx];
        self.morph_weights.clear();
        self.morph_weight_count = 0;

        if settings.is_playing {
            settings.time += dt * settings.speed;
            if settings.looping {
                if animation.duration > 0.0 {
                    settings.time %= animation.duration;
                }
            } else {
                settings.time = settings.time.clamp(0.0, animation.duration);
                if settings.time >= animation.duration {
                    settings.is_playing = false;
                }
            }
        }

        self.time = settings.time;
        self.speed = settings.speed;
        self.looping = settings.looping;
        self.is_playing = settings.is_playing;

        for channel in &animation.channels {
            let count = channel.times.len();
            if count == 0 {
                continue;
            }

            if count == 1 || settings.time <= channel.times[0] {
                Self::apply_single_keyframe(channel, 0, &mut self.local_pose, &mut self.morph_weights, model);
                continue;
            }
            if settings.time >= channel.times[count - 1] {
                Self::apply_single_keyframe(channel, count - 1, &mut self.local_pose, &mut self.morph_weights, model);
                continue;
            }

            let next_idx = channel.times.partition_point(|&t| t <= settings.time);
            let prev_idx = next_idx.saturating_sub(1);

            let start_time = channel.times[prev_idx];
            let end_time = channel.times[next_idx];
            let duration = end_time - start_time;

            let factor = if duration > 0.0 {
                (settings.time - start_time) / duration
            } else {
                0.0
            };

            let transform = self
                .local_pose
                .entry(channel.target_node)
                .or_insert_with(|| {
                    model
                        .nodes
                        .get(channel.target_node)
                        .map(|n| n.transform.clone())
                        .unwrap_or_else(NodeTransform::identity)
                });

            let dt = end_time - start_time;

            match &channel.values {
                ChannelValues::Translations(values) => {
                    transform.translation = match channel.interpolation {
                        AnimationInterpolation::Step => values[prev_idx],
                        AnimationInterpolation::Linear => {
                            let start = values[prev_idx];
                            let end = values[next_idx];
                            start.lerp(end, factor)
                        }
                        AnimationInterpolation::CubicSpline => {
                            let t = factor;
                            let t2 = t * t;
                            let t3 = t2 * t;

                            let idx0 = prev_idx * 3;
                            let idx1 = next_idx * 3;

                            if idx1 + 1 >= values.len() {
                                values[idx0 + 1]
                            } else {
                                let p0 = values[idx0 + 1];
                                let m0 = values[idx0 + 2] * dt;
                                let m1 = values[idx1 + 0] * dt;
                                let p1 = values[idx1 + 1];

                                let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                                let h10 = t3 - 2.0 * t2 + t;
                                let h01 = -2.0 * t3 + 3.0 * t2;
                                let h11 = t3 - t2;

                                p0 * h00 + m0 * h10 + p1 * h01 + m1 * h11
                            }
                        }
                    };
                }
                ChannelValues::Rotations(values) => {
                    transform.rotation = match channel.interpolation {
                        AnimationInterpolation::Step => values[prev_idx],
                        AnimationInterpolation::Linear => {
                            let start = values[prev_idx];
                            let end = values[next_idx];
                            start.slerp(end, factor).normalize()
                        }
                        AnimationInterpolation::CubicSpline => {
                            let t = factor;
                            let t2 = t * t;
                            let t3 = t2 * t;

                            let idx0 = prev_idx * 3;
                            let idx1 = next_idx * 3;

                            if idx1 + 1 >= values.len() {
                                values[idx0 + 1]
                            } else {
                                let p0 = values[idx0 + 1];
                                let m0 = values[idx0 + 2] * dt;
                                let m1 = values[idx1 + 0] * dt;
                                let p1 = values[idx1 + 1];

                                let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                                let h10 = t3 - 2.0 * t2 + t;
                                let h01 = -2.0 * t3 + 3.0 * t2;
                                let h11 = t3 - t2;

                                let res = p0 * h00 + m0 * h10 + p1 * h01 + m1 * h11;
                                res.normalize()
                            }
                        }
                    };
                }
                ChannelValues::Scales(values) => {
                    transform.scale = match channel.interpolation {
                        AnimationInterpolation::Step => values[prev_idx],
                        AnimationInterpolation::Linear => {
                            let start = values[prev_idx];
                            let end = values[next_idx];
                            start.lerp(end, factor)
                        }
                        AnimationInterpolation::CubicSpline => {
                            let t = factor;
                            let t2 = t * t;
                            let t3 = t2 * t;

                            let idx0 = prev_idx * 3;
                            let idx1 = next_idx * 3;

                            if idx1 + 1 >= values.len() {
                                values[idx0 + 1]
                            } else {
                                let p0 = values[idx0 + 1];
                                let m0 = values[idx0 + 2] * dt;
                                let m1 = values[idx1 + 0] * dt;
                                let p1 = values[idx1 + 1];

                                let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                                let h10 = t3 - 2.0 * t2 + t;
                                let h01 = -2.0 * t3 + 3.0 * t2;
                                let h11 = t3 - t2;

                                p0 * h00 + m0 * h10 + p1 * h01 + m1 * h11
                            }
                        }
                    };
                }
                ChannelValues::MorphWeights(values) => {
                    let weights = self.morph_weights
                        .entry(channel.target_node)
                        .or_insert_with(|| vec![0.0; values[0].len()]);

                    *weights = match channel.interpolation {
                        AnimationInterpolation::Step => values[prev_idx].clone(),
                        AnimationInterpolation::Linear => {
                            let a = &values[prev_idx];
                            let b = &values[next_idx];
                            a.iter().zip(b.iter())
                                .map(|(a, b)| a + (b - a) * factor)
                                .collect()
                        }
                        AnimationInterpolation::CubicSpline => {
                            // stored as [in_tangent, value, out_tangent] per keyframe
                            let p0 = &values[prev_idx * 3 + 1];
                            let m0 = &values[prev_idx * 3 + 2];
                            let m1 = &values[next_idx * 3 + 0];
                            let p1 = &values[next_idx * 3 + 1];
                            let t = factor;
                            let t2 = t * t;
                            let t3 = t2 * t;
                            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                            let h10 = t3 - 2.0 * t2 + t;
                            let h01 = -2.0 * t3 + 3.0 * t2;
                            let h11 = t3 - t2;
                            p0.iter().enumerate()
                                .map(|(i, p0i)| {
                                    p0i * h00 + m0[i] * dt * h10 + p1[i] * h01 + m1[i] * dt * h11
                                })
                                .collect()
                        }
                    };
                }
            }
        }

        self.update_matrices(model);
    }

    fn reset_to_bind_pose(&mut self, model: &Model) {
        self.local_pose.clear();
        self.morph_weights.clear();
        self.morph_weight_count = 0;
        self.update_matrices(model);
    }

    fn apply_single_keyframe(
        channel: &crate::model::AnimationChannel,
        index: usize,
        pose: &mut HashMap<usize, NodeTransform>,
        morph_weights: &mut HashMap<usize, Vec<f32>>,
        model: &Model,
    ) {
        let transform = pose.entry(channel.target_node).or_insert_with(|| {
            model
                .nodes
                .get(channel.target_node)
                .map(|n| n.transform.clone())
                .unwrap_or_else(NodeTransform::identity)
        });

        match &channel.values {
            ChannelValues::Translations(v) => {
                if let Some(val) = v.get(index) {
                    transform.translation = *val;
                }
            }
            ChannelValues::Rotations(v) => {
                if let Some(val) = v.get(index) {
                    transform.rotation = *val;
                }
            }
            ChannelValues::Scales(v) => {
                if let Some(val) = v.get(index) {
                    transform.scale = *val;
                }
            }
            ChannelValues::MorphWeights(v) => {
                let actual_index = match channel.interpolation {
                    AnimationInterpolation::CubicSpline => index * 3 + 1,
                    _ => index,
                };

                if let Some(frame) = v.get(actual_index) {
                    morph_weights.insert(channel.target_node, frame.clone());
                }
            },
        }
    }

    fn update_matrices(&mut self, model: &Model) {
        if let Some(skin) = model.skins.first() {
            if self.skinning_matrices.len() != skin.joints.len() {
                self.skinning_matrices
                    .resize(skin.joints.len(), Mat4::IDENTITY);
            }

            let mut global_transforms = HashMap::new();

            for &joint_idx in &skin.joints {
                self.resolve_global_transform(joint_idx, model, &mut global_transforms);
            }

            for (i, &joint_node_idx) in skin.joints.iter().enumerate() {
                if let Some(global_transform) = global_transforms.get(&joint_node_idx) {
                    let inverse_bind = skin.inverse_bind_matrices[i];
                    self.skinning_matrices[i] = *global_transform * inverse_bind;
                }
            }
        }
    }

    fn resolve_global_transform(
        &self,
        node_idx: usize,
        model: &Model,
        cache: &mut HashMap<usize, Mat4>,
    ) -> Mat4 {
        if let Some(&matrix) = cache.get(&node_idx) {
            return matrix;
        }

        let node = &model.nodes[node_idx];
        let local_matrix = self
            .local_pose
            .get(&node_idx)
            .map(|transform| transform.to_matrix())
            .unwrap_or_else(|| node.transform.to_matrix());

        let global_matrix = if let Some(parent_idx) = node.parent {
            let parent_global = self.resolve_global_transform(parent_idx, model, cache);
            parent_global * local_matrix
        } else {
            local_matrix
        };

        cache.insert(node_idx, global_matrix);
        global_matrix
    }

    pub fn prepare_gpu_resources(&mut self, graphics: Arc<SharedGraphicsContext>) {
        let has_skinning = !self.skinning_matrices.is_empty();
        let has_morph_weights = !self.morph_weights.is_empty();

        if !has_skinning && !has_morph_weights {
            return;
        }

        if has_skinning {
            let buffer = self.skinning_buffer.get_or_insert_with(|| {
                ResizableBuffer::new(
                    &graphics.device,
                    self.skinning_matrices.len(),
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    "skinning buffer",
                )
            });

            buffer.write(&graphics.device, &graphics.queue, &self.skinning_matrices);
        }

        if has_skinning || has_morph_weights {
            let mut flat: Vec<f32> = Vec::new();
            let mut num_targets: usize = 0;

            let mut sorted_nodes: Vec<usize> = self.morph_weights.keys().cloned().collect();
            sorted_nodes.sort();

            for weights in self.morph_weights.values() {
                num_targets = num_targets.max(weights.len());
            }

            if let Some(node_idx) = sorted_nodes.first() {
                let weights = &self.morph_weights[node_idx];
                flat.extend_from_slice(weights);
            }

            if flat.len() < num_targets {
                flat.resize(num_targets, 0.0);
            }

            if flat.is_empty() {
                flat.push(0.0);
            }

            let weights_buffer = self.morph_weights_buffer.get_or_insert_with(|| {
                ResizableBuffer::new(
                    &graphics.device,
                    flat.len().max(1),
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    "morph weights buffer",
                )
            });

            weights_buffer.write(&graphics.device, &graphics.queue, &flat);
            self.morph_weight_count = num_targets as u32;

            let info = MorphTargetInfo {
                num_vertices: 0,
                num_targets: num_targets as u32,
                base_offset: 0,
                weight_offset: 0,
                uses_morph: has_morph_weights as u32,
                _padding: Default::default(),
            };

            let info_buffer = self.morph_info_buffer.get_or_insert_with(|| {
                UniformBuffer::new(&graphics.device, "morph info buffer")
            });

            info_buffer.write(&graphics.queue, &info);

            self.morph_deltas_buffer = None;
        }
    }
}
