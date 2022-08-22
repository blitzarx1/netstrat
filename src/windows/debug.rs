use std::io::Write;
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender};
use egui::{ScrollArea, Ui, Widget, Window, CentralPanel, TopBottomPanel, Layout, TextEdit};
use egui_extras::{StripBuilder, Size};

use crate::AppWindow;

pub struct BuffWriter {
    pub publisher: Sender<Vec<u8>>,
}

impl Write for BuffWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.publisher
            .send(buf.into_iter().map(|el| *el).collect())
            .unwrap();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct Debug {
    filter: String,
    filtered: Vec<String>,
    buff: Vec<String>,
    max_lines: usize,
    receiver: Receiver<Vec<u8>>,
    visible: bool,
}

impl Debug {
    pub fn new(receiver: Receiver<Vec<u8>>, visible: bool, max_lines: usize) -> Self {
        let buff = vec![];
        let filtered = vec![];
        let filter = "".to_string();
        Self {
            filter,
            filtered,
            buff,
            max_lines,
            receiver,
            visible,
        }
    }

    fn update_data(&mut self) {
        let data_wrapped = self.receiver.recv_timeout(Duration::from_millis(1));
        match data_wrapped {
            Ok(data) => {
                self.add_new_message(data);
            }
            Err(_) => {}
        }
    }

    fn add_new_message(&mut self, msg: Vec<u8>) {
        if self.buff.len() > self.max_lines {
            if let Some(split_res) = self.buff.split_first() {
                self.buff = split_res.1.to_vec();
            }
        }
        self.buff
            .push(String::from_utf8_lossy(msg.as_slice()).to_string());

        self.filtered = self.buff.iter().filter(|el| {
                el.to_lowercase().contains(&self.filter)
            }).map(|el|el.clone()).collect();
    }
}

impl AppWindow for Debug {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("debug").clicked() {
            self.visible = !self.visible
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        self.update_data();

        Window::new("debug")
            .open(&mut self.visible)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui|{
                    if TextEdit::singleline(&mut self.filter)
                    .hint_text("filter")
                    .show(ui)
                    .response
                    .changed() {
                        self.filtered = self.buff.iter().filter(|el| {
                            el.to_lowercase().contains(&self.filter)
                        }).map(|el|el.clone()).collect();

                    }
                    ui.label(format!("{}/{}", self.filtered.len(), self.buff.len()))
                });

                ScrollArea::new([true, true])
                .stick_to_bottom()
                .show(ui, |ui| {
                    self.filtered.iter().for_each(|line| {
                        egui::widgets::Label::new(line).wrap(false).ui(ui);
                    });
                });
            });         
    }
}
