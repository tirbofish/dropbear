use crate::{model::ModelVertex, procedural::ProcedurallyGeneratedObject};

impl ProcedurallyGeneratedObject {
    /// Creates a cuboid (box) procedurally.
    ///
    /// `size` is the full extents (width, height, depth).
    pub fn cuboid(size: glam::DVec3) -> Self {
        let half = (size / 2.0).as_vec3();

        let uv_x = 1.0_f32;
        let uv_y = 1.0_f32;
        let uv_z = 1.0_f32;

        let make_vertex = |position: [f32; 3], normal: [f32; 3], tangent: [f32; 3], uv: [f32; 2]| {
            ModelVertex {
                position,
                normal,
                tangent: [tangent[0], tangent[1], tangent[2], 1.0],
                tex_coords0: uv,
                tex_coords1: [0.0, 0.0],
                colour0: [1.0, 1.0, 1.0, 1.0],
                joints0: [0, 0, 0, 0],
                weights0: [1.0, 0.0, 0.0, 0.0],
            }
        };

        let vertices = vec![
            // Front Face (Normal: 0, 0, 1)
            make_vertex([-half.x, -half.y,  half.z], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0,  uv_y]),
            make_vertex([ half.x, -half.y,  half.z], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [uv_x, uv_y]),
            make_vertex([ half.x,  half.y,  half.z], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [uv_x, 0.0]),
            make_vertex([-half.x,  half.y,  half.z], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0,  0.0]),

            // Back Face (Normal: 0, 0, -1)
            make_vertex([ half.x, -half.y, -half.z], [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0,  uv_y]),
            make_vertex([-half.x, -half.y, -half.z], [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [uv_x, uv_y]),
            make_vertex([-half.x,  half.y, -half.z], [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [uv_x, 0.0]),
            make_vertex([ half.x,  half.y, -half.z], [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0,  0.0]),

            // Top Face (Normal: 0, 1, 0)
            make_vertex([-half.x,  half.y,  half.z], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0,  uv_z]),
            make_vertex([ half.x,  half.y,  half.z], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [uv_x, uv_z]),
            make_vertex([ half.x,  half.y, -half.z], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [uv_x, 0.0]),
            make_vertex([-half.x,  half.y, -half.z], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0,  0.0]),

            // Bottom Face (Normal: 0, -1, 0)
            make_vertex([-half.x, -half.y, -half.z], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0,  uv_z]),
            make_vertex([ half.x, -half.y, -half.z], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [uv_x, uv_z]),
            make_vertex([ half.x, -half.y,  half.z], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [uv_x, 0.0]),
            make_vertex([-half.x, -half.y,  half.z], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0,  0.0]),

            // Right Face (Normal: 1, 0, 0)
            make_vertex([ half.x, -half.y,  half.z], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0,  uv_y]),
            make_vertex([ half.x, -half.y, -half.z], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [uv_z, uv_y]),
            make_vertex([ half.x,  half.y, -half.z], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [uv_z, 0.0]),
            make_vertex([ half.x,  half.y,  half.z], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0,  0.0]),

            // Left Face (Normal: -1, 0, 0)
            make_vertex([-half.x, -half.y, -half.z], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0,  uv_y]),
            make_vertex([-half.x, -half.y,  half.z], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [uv_z, uv_y]),
            make_vertex([-half.x,  half.y,  half.z], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [uv_z, 0.0]),
            make_vertex([-half.x,  half.y, -half.z], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0,  0.0]),
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,    // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self { vertices, indices }
    }
}