pub struct UiEditor {
    pub active_entity: Option<hecs::Entity>,
}

impl UiEditor {
    pub fn new() -> Self {
        Self {
            active_entity: None,
        }
    }

    pub fn update(&mut self) {

    }
}