use egui::{ColorImage, ScrollArea, TextureHandle};

use crate::widgets::AppWidget;

use super::image_state::ImageState;

#[derive(Default, Clone)]
pub struct NetVisualizer {
    current_image: ImageState,
    current_texture: Option<TextureHandle>,
}

impl NetVisualizer {
    pub fn update_image(&mut self, image: ColorImage) {
        self.current_image.update(image);
    }

    pub fn changed(&mut self) -> bool {
        self.current_image.changed()
    }
}

impl AppWidget for NetVisualizer {
    fn show(&mut self, ui: &mut egui::Ui) {
        if self.current_image.changed() {
            self.current_texture = Some(ui.ctx().load_texture(
                "net",
                self.current_image.image(),
                egui::TextureFilter::Linear,
            ))
        }

        let texture = self.current_texture.clone().unwrap();
        ScrollArea::both()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.add(egui::Image::new(&texture, texture.size_vec2()));
            });
    }
}
