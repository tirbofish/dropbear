/// Different input signatures for Native implementations
use std::ffi::c_char;
use crate::scripting::DropbearContext;

/// CName: `dropbear_init`
pub type Init = unsafe extern "C" fn(dropbear_context: *const DropbearContext) -> i32;
/// CName: `dropbear_load_systems`
pub type LoadTagged = unsafe extern "C" fn(tag: *const c_char) -> i32;

/// CName: `dropbear_load_with_entities`
pub type LoadWithEntities = unsafe extern "C" fn(
    tag: *const c_char,
    entities: *const u64,
    entity_count: i32,
) -> i32;
/// CName: `dropbear_update_all`
pub type UpdateAll = unsafe extern "C" fn(dt: f64) -> i32;
/// CName: `dropbear_update_tagged`
pub type UpdateTagged = unsafe extern "C" fn(tag: *const c_char, dt: f64) -> i32;
/// CName: `dropbear_update_with_entities`
pub type UpdateWithEntities = unsafe extern "C" fn(
    tag: *const c_char,
    entities: *const u64,
    entity_count: i32,
    dt: f64
) -> i32;

/// CName: `dropbear_physics_update_all`
pub type PhysicsUpdateAll = unsafe extern "C" fn(dt: f64) -> i32;
/// CName: `dropbear_physics_update_tagged`
pub type PhysicsUpdateTagged = unsafe extern "C" fn(tag: *const c_char, dt: f64) -> i32;
/// CName: `dropbear_physics_update_with_entities`
pub type PhysicsUpdateWithEntities = unsafe extern "C" fn(
    tag: *const c_char,
    entities: *const u64,
    entity_count: i32,
    dt: f64
) -> i32;

/// CName: `dropbear_collision_event`
pub type CollisionEvent = unsafe extern "C" fn(
    tag: *const c_char,
    current_entity_id: u64,
    event_type: i32,
    c1_index: i32,
    c1_generation: i32,
    c1_entity_id: u64,
    c1_id: i32,
    c2_index: i32,
    c2_generation: i32,
    c2_entity_id: u64,
    c2_id: i32,
    flags: u64,
) -> i32;

/// CName: `dropbear_contact_force_event`
pub type ContactForceEvent = unsafe extern "C" fn(
    tag: *const c_char,
    current_entity_id: u64,
    c1_index: i32,
    c1_generation: i32,
    c1_entity_id: u64,
    c1_id: i32,
    c2_index: i32,
    c2_generation: i32,
    c2_entity_id: u64,
    c2_id: i32,
    total_fx: f64,
    total_fy: f64,
    total_fz: f64,
    total_force_magnitude: f64,
    max_fx: f64,
    max_fy: f64,
    max_fz: f64,
    max_force_magnitude: f64,
) -> i32;
/// CName: `dropbear_destroy_tagged`
pub type DestroyTagged = unsafe extern "C" fn(tag: *const c_char) -> i32;
/// CName: `dropbear_destroy_in_scope_tagged`
pub type DestroyInScopeTagged = unsafe extern "C" fn(tag: *const c_char) -> i32;
/// CName: `dropbear_destroy_all`
pub type DestroyAll = unsafe extern "C" fn() -> i32;

/// CName: `dropbear_get_last_error_message`
pub type GetLastErrorMessage = unsafe extern "C" fn() -> *const c_char;
/// CName: `dropbear_set_last_error_message`
pub type SetLastErrorMessage = unsafe extern "C" fn(msg: *const c_char);

