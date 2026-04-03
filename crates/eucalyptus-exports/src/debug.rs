use eucalyptus_core::ptr::GraphicsContextPtr;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::types::{NColour, NVector3};
use crate::math::NQuaternion;
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
fn draw_line(
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
fn draw_ray(
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
fn draw_arrow(
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
fn draw_point(
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
fn draw_circle(
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
fn draw_sphere(
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
fn draw_globe(
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
fn draw_aabb(
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
fn draw_obb(
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
fn draw_capsule(
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
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawCylinder"),
    c
)]
fn draw_cylinder(
    #[dropbear_macro::define(GraphicsContextPtr)] graphics: &SharedGraphicsContext,
    center: &NVector3,
    half_height: f32,
    radius: f32,
    axis: &NVector3,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let c = Vec3::new(center.x as f32, center.y as f32, center.z as f32);
    let ax = Vec3::new(axis.x as f32, axis.y as f32, axis.z as f32);
    with_debug!(graphics, |dd| dd.draw_cylinder(c, half_height, radius, ax, colour.to_f32_array()));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.rendering.DebugDrawNative", func = "drawCone"),
    c
)]
fn draw_cone(
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
