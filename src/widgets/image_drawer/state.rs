use egui::ColorImage;

#[derive(Default, Clone)]
pub struct State {
    is_read: bool,
    image: ColorImage,
}

impl State {
    pub fn update(&mut self, new_image: ColorImage) {
        self.image = new_image;
        self.is_read = false;
    }

    pub fn is_read(&self) -> bool {
        self.is_read
    }

    pub fn read(&mut self) -> &ColorImage {
        self.is_read = true;
        &self.image
    }
}
