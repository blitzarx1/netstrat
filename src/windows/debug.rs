use std::io::Write;
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender};
use egui::{ScrollArea, TextEdit, Ui, Window};
use tracing::{debug, info};

use crate::{netstrat::line_filter_highlight_layout::line_filter_highlight_layout, AppWindow};

pub struct BuffWriter {
    pub publisher: Sender<Vec<u8>>,
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

    fn update_data(&mut self, filter: String, visible: bool) {
        loop {
            let data_wrapped = self.receiver.recv_timeout(Duration::from_millis(1));
            match data_wrapped {
                Ok(data) => self.add_new_message(data),
                Err(_) => break,
            }
        }

        self.apply_filter(filter);

        self.visible = visible;
    }

    fn add_new_message(&mut self, msg: Vec<u8>) {
        self.handle_size_limit();

        let msg_text = String::from_utf8_lossy(msg.as_slice()).to_string();
        self.buff.push(msg_text.clone());
        if msg_text
            .to_lowercase()
            .contains(&self.filter.to_lowercase())
        {
            self.filtered.push(msg_text)
        }
    }

    fn handle_size_limit(&mut self) {
        if self.buff.len() > self.max_lines {
            remove_first_line(&mut self.buff)
        }

        if self.filtered.len() > self.max_lines {
            remove_first_line(&mut self.filtered)
        }
    }

    fn apply_filter(&mut self, new_filter: String) {
        let filter_normalized = new_filter.to_lowercase();
        if filter_normalized == self.filter {
            return;
        }

        info!("applying filter");
        debug!("filter: {new_filter}");

        // optimization
        if filter_normalized.contains(self.filter.as_str()) {
            debug!("using optimized version");

            self.filtered = self
                .filtered
                .iter()
                .filter(|el| el.to_lowercase().contains(filter_normalized.as_str()))
                .cloned()
                .collect();

            self.filter = filter_normalized;
            return;
        }

        debug!("using heavy version");

        self.filtered = self
            .buff
            .iter()
            .filter(|el| el.to_lowercase().contains(filter_normalized.as_str()))
            .cloned()
            .collect();

        self.filter = filter_normalized;
    }
}

impl Write for BuffWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.publisher.send(buf.to_vec()).unwrap();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl AppWindow for Debug {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("debug").clicked() {
            self.visible = !self.visible
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let mut filter = self.filter.clone();
        let mut visible = self.visible;
        Window::new("debug")
            .open(&mut visible)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    TextEdit::singleline(&mut filter)
                        .hint_text("filter")
                        .show(ui);
                    ui.label(format!("{}/{}", self.filtered.len(), self.buff.len()))
                });

                ui.add_space(10.0);

                ScrollArea::new([true, true])
                    .stick_to_bottom()
                    .show(ui, |ui| {
                        let mut lines = self.filtered.concat();
                        let mut layouter = |ui: &egui::Ui, string: &str, _: f32| {
                            ui.fonts().layout_job(line_filter_highlight_layout(
                                ui, string, &filter, false,
                            ))
                        };

                        TextEdit::multiline(&mut lines)
                            .layouter(&mut layouter)
                            .show(ui);
                    });
            });

        self.update_data(filter, visible);
    }
}

fn remove_first_line(lines: &mut Vec<String>) {
    if let Some(split_res) = lines.split_first() {
        *lines = split_res.1.to_vec();
    }
}
