use egui::ColorImage;

#[derive(Default, Clone)]
pub struct ImageState {
    changed: bool,
    image: ColorImage,
}

impl ImageState {
    pub fn new(image: ColorImage) -> Self {
        Self {
            changed: true,
            image,
        }
    }

    pub fn update(&mut self, new_image: ColorImage) {
        self.image = new_image;
        self.changed = true;
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn image(&mut self) -> ColorImage {
        self.changed = false;
        self.image.clone()
    }
}
