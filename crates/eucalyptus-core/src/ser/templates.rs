/// A template that can be used to display entities and their children.
///
/// Contains all the assets required. On final compilation, it will be resolved, 
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, serde::Serialize, serde::Deserialize)]
pub struct Template {
    pub label: String,
}

impl Template {
    pub fn new(label: String) -> Self {
        Self { label }
    }

    pub fn update(&mut self, ) {

    }
}