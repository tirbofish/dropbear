use crate::ptr::GraphicsContextPtr;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{NColour, NQuaternion, NVector3};
use dropbear_engine::graphics::SharedGraphicsContext;
use glam::{Quat, Vec3};

macro_rules! with_debug {
    ($graphics:expr, |$dd:ident| $body:expr) => {
        if let Some($dd) = $graphics.debug_draw.lock().as_mut() {
            $body
        }
    };
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawLine"),
    c
)]
fn debug_draw_line(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    start: &NVector3,
    end: &NVector3,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let a = Vec3::new(start.x as f32, start.y as f32, start.z as f32);
    let b = Vec3::new(end.x as f32, end.y as f32, end.z as f32);
    with_debug!(graphics, |dd| dd.draw_line(a, b, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawRay"),
    c
)]
fn debug_draw_ray(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    origin: &NVector3,
    dir: &NVector3,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let o = Vec3::new(origin.x as f32, origin.y as f32, origin.z as f32);
    let d = Vec3::new(dir.x as f32, dir.y as f32, dir.z as f32);
    with_debug!(graphics, |dd| dd.draw_ray(o, d, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawArrow"),
    c
)]
fn debug_draw_arrow(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    start: &NVector3,
    end: &NVector3,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let a = Vec3::new(start.x as f32, start.y as f32, start.z as f32);
    let b = Vec3::new(end.x as f32, end.y as f32, end.z as f32);
    with_debug!(graphics, |dd| dd.draw_arrow(a, b, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawPoint"),
    c
)]
fn debug_draw_point(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    pos: &NVector3,
    size: f32,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let p = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
    with_debug!(graphics, |dd| dd.draw_point(p, size, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawCircle"),
    c
)]
fn debug_draw_circle(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    center: &NVector3,
    radius: f32,
    normal: &NVector3,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let c = Vec3::new(center.x as f32, center.y as f32, center.z as f32);
    let n = Vec3::new(normal.x as f32, normal.y as f32, normal.z as f32);
    with_debug!(graphics, |dd| dd.draw_circle(c, radius, n, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawSphere"),
    c
)]
fn debug_draw_sphere(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    center: &NVector3,
    radius: f32,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let c = Vec3::new(center.x as f32, center.y as f32, center.z as f32);
    with_debug!(graphics, |dd| dd.draw_sphere(c, radius, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawGlobe"),
    c
)]
fn debug_draw_globe(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    center: &NVector3,
    radius: f32,
    lat_lines: u32,
    lon_lines: u32,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let c = Vec3::new(center.x as f32, center.y as f32, center.z as f32);
    with_debug!(graphics, |dd| dd.draw_globe(c, radius, lat_lines, lon_lines, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawAabb"),
    c
)]
fn debug_draw_aabb(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    min: &NVector3,
    max: &NVector3,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let mn = Vec3::new(min.x as f32, min.y as f32, min.z as f32);
    let mx = Vec3::new(max.x as f32, max.y as f32, max.z as f32);
    with_debug!(graphics, |dd| dd.draw_aabb(mn, mx, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawObb"),
    c
)]
fn debug_draw_obb(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    center: &NVector3,
    half_extents: &NVector3,
    rotation: &NQuaternion,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let c = Vec3::new(center.x as f32, center.y as f32, center.z as f32);
    let he = Vec3::new(half_extents.x as f32, half_extents.y as f32, half_extents.z as f32);
    let r = Quat::from_xyzw(rotation.x as f32, rotation.y as f32, rotation.z as f32, rotation.w as f32);
    with_debug!(graphics, |dd| dd.draw_obb(c, he, r, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawCapsule"),
    c
)]
fn debug_draw_capsule(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    a: &NVector3,
    b: &NVector3,
    radius: f32,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let va = Vec3::new(a.x as f32, a.y as f32, a.z as f32);
    let vb = Vec3::new(b.x as f32, b.y as f32, b.z as f32);
    with_debug!(graphics, |dd| dd.draw_capsule(va, vb, radius, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawCone"),
    c
)]
fn debug_draw_cone(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    apex: &NVector3,
    dir: &NVector3,
    angle: f32,
    length: f32,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let a = Vec3::new(apex.x as f32, apex.y as f32, apex.z as f32);
    let d = Vec3::new(dir.x as f32, dir.y as f32, dir.z as f32);
    with_debug!(graphics, |dd| dd.draw_cone(a, d, angle, length, colour.to_f32_array()));
    Ok(())
}
