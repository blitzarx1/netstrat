use std::sync::{Mutex, Arc};

use crate::{widgets::NetProps, netstrat::Drawer};
use crossbeam::channel::Sender;
use egui::{ScrollArea, Ui, Window};
use tracing::info;

use super::AppWindow;
use crate::widgets::AppWidget;

pub struct Net {
    visible: bool,
    net: NetProps,
}

impl AppWindow for Net {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("net").clicked() {
            self.update_visible(!self.visible)
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let mut visible = self.visible;

        Window::new("net").open(&mut visible).show(ui.ctx(), |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.net.show(ui);
            })
        });

        self.update_visible(visible);
    }
}

impl Net {
    pub fn new(widget_pub: Sender<Arc<Mutex<Box<dyn Drawer>>>>, visible: bool) -> Self {
        info!("initing window net");
        Self {
            net: NetProps::new(widget_pub),
            visible,
        }
    }

    fn update_visible(&mut self, visible: bool) {
        if visible != self.visible {
            self.visible = visible;
            match visible {
                true => info!("opening net window..."),
                false => info!("closing net window..."),
            }
        }
    }
}
