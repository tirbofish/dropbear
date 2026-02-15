use std::sync::Arc;

use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::model::Model;
use dropbear_engine::procedural::ProcedurallyGeneratedObject;
use dropbear_engine::utils::{ResourceReference, ResourceReferenceType};

use crate::states::SerializedMeshRenderer;
use crate::utils::ResolveReference;

pub async fn load_mesh_renderer_from_serialized(
    serialized: &SerializedMeshRenderer,
    graphics: Arc<SharedGraphicsContext>,
    label: &str,
) -> anyhow::Result<MeshRenderer> {
    let import_scale = serialized.import_scale.unwrap_or(1.0);

    let handle = match &serialized.handle.ref_type {
        ResourceReferenceType::None => {
            anyhow::bail!("Resource reference type is None for '{}'", label);
        }
        ResourceReferenceType::Unassigned { id } => {
            let model = Model {
                label: "None".to_string(),
                hash: *id,
                path: ResourceReference::from_reference(ResourceReferenceType::Unassigned { id: *id }),
                meshes: Vec::new(),
                materials: Vec::new(),
                skins: Vec::new(),
                animations: Vec::new(),
                nodes: Vec::new(),
            };

            let mut registry = ASSET_REGISTRY.write();
            if let Some(existing) = registry.model_handle_by_hash(*id) {
                existing
            } else {
                registry.add_model(model)
            }
        }
        ResourceReferenceType::File(reference) => {
            let path = serialized.handle.resolve()?;
            log::debug!("Path for entity {} is {} from reference {}", label, path.display(), reference);
            let buffer = std::fs::read(&path)?;
            Model::load_from_memory_raw(graphics.clone(), buffer, Some(label), ASSET_REGISTRY.clone()).await?
        }
        ResourceReferenceType::Bytes(bytes) => {
            log::info!("Loading entity '{}' from bytes [Len: {}]", label, bytes.len());
            Model::load_from_memory_raw(graphics.clone(), bytes, Some(label), ASSET_REGISTRY.clone()).await?
        }
        ResourceReferenceType::ProcObj(obj) => match obj {
            dropbear_engine::procedural::ProcObj::Cuboid { size_bits } => {
                let size = [
                    f32::from_bits(size_bits[0]),
                    f32::from_bits(size_bits[1]),
                    f32::from_bits(size_bits[2]),
                ];
                log::info!("Loading entity '{}' from cuboid: {:?}", label, size);

                let size_vec = glam::DVec3::new(size[0] as f64, size[1] as f64, size[2] as f64);
                ProcedurallyGeneratedObject::cuboid(size_vec).build_model(
                    graphics.clone(),
                    None,
                    Some(label),
                    ASSET_REGISTRY.clone(),
                )
            }
        },
    };

    let mut renderer = MeshRenderer::from_handle(handle);
    renderer.set_import_scale(import_scale);

    if serialized.texture_override.is_some() {
        log::debug!(
            "texture_override is set for '{}' but not applied yet", 
            label
        );
    }

    Ok(renderer)
}
