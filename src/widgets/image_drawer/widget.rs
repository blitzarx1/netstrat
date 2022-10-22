use egui::{ColorImage, ScrollArea, TextureHandle};

use crate::{netstrat::Drawer, widgets::AppWidget};

use super::state::State;

#[derive(Default, Clone)]
pub struct ImageDrawer {
    current_image: State,
    current_texture: Option<TextureHandle>,
}

impl Drawer for ImageDrawer {
    fn update_image(&mut self, image: ColorImage) {
        self.current_image.update(image);
    }

    fn has_unread_image(&self) -> bool {
        !self.current_image.is_read()
    }
}

impl AppWidget for ImageDrawer {
    fn show(&mut self, ui: &mut egui::Ui) {
        if !self.current_image.is_read() {
            let new_image = self.current_image.read().clone();
            self.current_texture = Some(ui.ctx().load_texture(
                "net",
                new_image,
                egui::TextureFilter::Linear,
            ));
        }

        let texture = self.current_texture.clone().unwrap();
        ScrollArea::both()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.add(egui::Image::new(&texture, texture.size_vec2()));
            });
    }
}
