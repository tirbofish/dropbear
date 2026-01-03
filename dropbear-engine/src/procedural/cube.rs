use crate::{model::ModelVertex, procedural::ProcedurallyGeneratedObject};

impl ProcedurallyGeneratedObject {
    pub fn cuboid(size: glam::DVec3) -> Self {
        let half = (size / 2.0).as_vec3();

        let uv_x = size.x as f32;
        let uv_y = size.y as f32;
        let uv_z = size.z as f32;

        let vertices = vec![
            // Front Face (Normal: 0, 0, 1)
            ModelVertex { position: [-half.x, -half.y,  half.z], tex_coords: [0.0,  uv_y], normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [ half.x, -half.y,  half.z], tex_coords: [uv_x, uv_y], normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [ half.x,  half.y,  half.z], tex_coords: [uv_x, 0.0],  normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [-half.x,  half.y,  half.z], tex_coords: [0.0,  0.0],  normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },

            // Back Face (Normal: 0, 0, -1)
            ModelVertex { position: [ half.x, -half.y, -half.z], tex_coords: [0.0,  uv_y], normal: [0.0, 0.0, -1.0], tangent: [-1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [-half.x, -half.y, -half.z], tex_coords: [uv_x, uv_y], normal: [0.0, 0.0, -1.0], tangent: [-1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [-half.x,  half.y, -half.z], tex_coords: [uv_x, 0.0],  normal: [0.0, 0.0, -1.0], tangent: [-1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [ half.x,  half.y, -half.z], tex_coords: [0.0,  0.0],  normal: [0.0, 0.0, -1.0], tangent: [-1.0, 0.0, 0.0], bitangent: [0.0, -1.0, 0.0] },

            // Top Face (Normal: 0, 1, 0)
            ModelVertex { position: [-half.x,  half.y,  half.z], tex_coords: [0.0,  uv_z], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, 1.0] },
            ModelVertex { position: [ half.x,  half.y,  half.z], tex_coords: [uv_x, uv_z], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, 1.0] },
            ModelVertex { position: [ half.x,  half.y, -half.z], tex_coords: [uv_x, 0.0],  normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, 1.0] },
            ModelVertex { position: [-half.x,  half.y, -half.z], tex_coords: [0.0,  0.0],  normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, 1.0] },

            // Bottom Face (Normal: 0, -1, 0)
            ModelVertex { position: [-half.x, -half.y, -half.z], tex_coords: [0.0,  uv_z], normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, -1.0] },
            ModelVertex { position: [ half.x, -half.y, -half.z], tex_coords: [uv_x, uv_z], normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, -1.0] },
            ModelVertex { position: [ half.x, -half.y,  half.z], tex_coords: [uv_x, 0.0],  normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, -1.0] },
            ModelVertex { position: [-half.x, -half.y,  half.z], tex_coords: [0.0,  0.0],  normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, -1.0] },

            // Right Face (Normal: 1, 0, 0)
            ModelVertex { position: [ half.x, -half.y,  half.z], tex_coords: [0.0,  uv_y], normal: [1.0, 0.0, 0.0], tangent: [0.0, 0.0, -1.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [ half.x, -half.y, -half.z], tex_coords: [uv_z, uv_y], normal: [1.0, 0.0, 0.0], tangent: [0.0, 0.0, -1.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [ half.x,  half.y, -half.z], tex_coords: [uv_z, 0.0],  normal: [1.0, 0.0, 0.0], tangent: [0.0, 0.0, -1.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [ half.x,  half.y,  half.z], tex_coords: [0.0,  0.0],  normal: [1.0, 0.0, 0.0], tangent: [0.0, 0.0, -1.0], bitangent: [0.0, -1.0, 0.0] },

            // Left Face (Normal: -1, 0, 0)
            ModelVertex { position: [-half.x, -half.y, -half.z], tex_coords: [0.0,  uv_y], normal: [-1.0, 0.0, 0.0], tangent: [0.0, 0.0, 1.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [-half.x, -half.y,  half.z], tex_coords: [uv_z, uv_y], normal: [-1.0, 0.0, 0.0], tangent: [0.0, 0.0, 1.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [-half.x,  half.y,  half.z], tex_coords: [uv_z, 0.0],  normal: [-1.0, 0.0, 0.0], tangent: [0.0, 0.0, 1.0], bitangent: [0.0, -1.0, 0.0] },
            ModelVertex { position: [-half.x,  half.y, -half.z], tex_coords: [0.0,  0.0],  normal: [-1.0, 0.0, 0.0], tangent: [0.0, 0.0, 1.0], bitangent: [0.0, -1.0, 0.0] },
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,    // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self {
            vertices,
            indices
        }
    }
}