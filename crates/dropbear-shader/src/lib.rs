use wesl::include_wesl;

wesl::wesl_pkg!(dropbear);

pub const LIGHT_SHADER: &str = include_wesl!("dropbear_light");
pub const SHADER_SHADER: &str = include_wesl!("dropbear_shader");
pub const SHADOW_SHADER: &str = include_wesl!("dropbear_shadow");
pub const OUTLINE_SHADER: &str = include_wesl!("dropbear_outline");
pub const COLLIDER_SHADER: &str = include_wesl!("dropbear_collider");
pub const MIPMAP_SHADER: &str = include_wesl!("dropbear_mipmap");