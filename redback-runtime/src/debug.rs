use std::any::TypeId;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{MeshRenderer, Transform};
use dropbear_engine::lighting::{Light, LightComponent};
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::states::{Label, CustomProperties, Script};

impl Runtime {
    #[allow(dead_code)]
    pub fn display_all_entities(&self) {
        log::debug!("====================");
        log::info!("world total items: {}", self.world.len());
        log::info!("typeid of Label: {:?}", TypeId::of::<Label>());
        log::info!("typeid of MeshRenderer: {:?}", TypeId::of::<MeshRenderer>());
        log::info!("typeid of Transform: {:?}", TypeId::of::<Transform>());
        log::info!(
                    "typeid of ModelProperties: {:?}",
                    TypeId::of::<CustomProperties>()
                );
        log::info!("typeid of Camera: {:?}", TypeId::of::<Camera>());
        log::info!(
                    "typeid of CameraComponent: {:?}",
                    TypeId::of::<CameraComponent>()
                );
        log::info!("typeid of Script: {:?}", TypeId::of::<Script>());
        log::info!("typeid of Light: {:?}", TypeId::of::<Light>());
        log::info!(
                    "typeid of LightComponent: {:?}",
                    TypeId::of::<LightComponent>()
                );
        for i in self.world.iter() {
            log::info!("entity id: {:?}", i.entity().id());
            log::info!("entity bytes: {:?}", i.entity().to_bits().get());
            log::info!(
                        "components [{}]: ",
                        i.component_types().collect::<Vec<_>>().len()
                    );
            let mut comp_builder = String::new();
            for j in i.component_types() {
                comp_builder.push_str(format!("{:?} ", j).as_str());
                if TypeId::of::<Label>() == j {
                    log::info!(" |- Label");
                }

                if TypeId::of::<MeshRenderer>() == j {
                    log::info!(" |- MeshRenderer");
                }

                if TypeId::of::<Transform>() == j {
                    log::info!(" |- Transform");
                }

                if TypeId::of::<CustomProperties>() == j {
                    log::info!(" |- ModelProperties");
                }

                if TypeId::of::<Camera>() == j {
                    log::info!(" |- Camera");
                }

                if TypeId::of::<CameraComponent>() == j {
                    log::info!(" |- CameraComponent");
                }

                if TypeId::of::<Script>() == j {
                    log::info!(" |- Script");
                }

                if TypeId::of::<Light>() == j {
                    log::info!(" |- Light");
                }

                if TypeId::of::<LightComponent>() == j {
                    log::info!(" |- LightComponent");
                }
                log::info!("----------")
            }
            log::info!("components (typeid) [{}]: ", comp_builder);
        }
    }
}