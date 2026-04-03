use eucalyptus_core::hierarchy::{Children, Parent};
use eucalyptus_core::ptr::WorldPtr;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::states::Label;

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.LabelNative", func = "labelExistsForEntity"),
    c
)]
fn label_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&Label>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.EntityRefNative", func = "getEntityLabel"),
    c
)]
fn get_label(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<String> {
    let label = world.get::<&Label>(entity)?.as_str().to_string();
    Ok(label)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.EntityRefNative", func = "getChildren"),
    c
)]
fn get_children(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<Vec<u64>> {
    if let Ok(children) = world.query_one::<&Children>(entity).get() {
        let entity_bytes = children
            .children()
            .iter()
            .map(|c| c.to_bits().get())
            .collect::<Vec<_>>();
        Ok(entity_bytes)
    } else {
        Ok(vec![])
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.EntityRefNative", func = "getChildByLabel"),
    c
)]
fn get_child_by_label(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    target: String,
) -> DropbearNativeResult<Option<u64>> {
    if let Ok(children) = world.query_one::<&Children>(entity).get() {
        for child in children.children() {
            if let Ok(label) = world.get::<&Label>(*child) {
                if label.as_str() == target {
                    let found = child.clone();
                    return Ok(Some(found.to_bits().get()));
                }
            } else {
                continue;
            }
        }
        Ok(None)
    } else {
        Ok(None)
    }
}

#[dropbear_macro::export(kotlin(class = "com.dropbear.EntityRefNative", func = "getParent"), c)]
fn get_parent(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<Option<u64>> {
    if let Ok(parent) = world.query_one::<&Parent>(entity).get() {
        Ok(Some(parent.parent().to_bits().get()))
    } else {
        Ok(None)
    }
}
