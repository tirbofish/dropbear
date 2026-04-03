use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::scripting::types::KotlinComponents;
use hecs::{Entity, World};

#[dropbear_macro::export(kotlin(
    class = "com.dropbear.components.ComponentNative",
    func = "hasKotlinComponent",
))]
fn has_kotlin_component(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    fqcn: String,
) -> DropbearNativeResult<bool> {
    if let Ok(kc) = world.get::<&KotlinComponents>(entity) {
        Ok(kc.has(&fqcn))
    } else {
        Ok(false)
    }
}

#[dropbear_macro::export(kotlin(
    class = "com.dropbear.components.ComponentNative",
    func = "registerKotlinComponent",
))]
fn register_kotlin_component_jni(
    #[dropbear_macro::define(WorldPtr)] _world: &World,
    fqcn: String,
    type_name: String,
    category: String,
    description: String,
) -> DropbearNativeResult<()> {
    use eucalyptus_core::component::{KOTLIN_COMPONENT_QUEUE, KotlinComponentDecl};
    KOTLIN_COMPONENT_QUEUE.lock().push(KotlinComponentDecl {
        fqcn,
        type_name,
        category: if category.is_empty() { None } else { Some(category) },
        description: if description.is_empty() { None } else { Some(description) },
    });
    Ok(())
}
