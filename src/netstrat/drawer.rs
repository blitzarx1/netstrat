use egui::ColorImage;

use crate::widgets::AppWidget;

pub trait Drawer: AppWidget {
    fn update_image(&mut self, image: ColorImage);
    fn has_unread_image(&self) -> bool;
}
