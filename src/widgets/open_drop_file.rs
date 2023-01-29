use egui::{CursorIcon, FontSelection, Sense, TextEdit, TextStyle};
use tracing::debug;

use super::AppWidget;

const HINT: &str = "Drop a .dot file here or click to open a file dialog";

#[derive(Default)]
pub struct OpenDropFile {
    file_path: Option<String>,
}

impl OpenDropFile {
    pub fn path(&mut self) -> Option<String> {
        let path = self.file_path.clone();

        if self.file_path.is_some() {
            self.file_path = None;
            debug!("file path resetted to None");
        }

        path
    }

    fn update(&mut self, file_path: String) {
        if !file_path.is_empty() {
            self.file_path = Some(file_path.clone());
            debug!("file path updated: {file_path:?}");
        }
    }
}

impl AppWidget for OpenDropFile {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut file_path = "".to_string();
        if let Some(path) = self.file_path.clone() {
            file_path = path;
        }

        let mut text = HINT.to_string();
        if !ui.ctx().input().raw.hovered_files.is_empty() {
            text = format!(
                "Dropping file: {:?}",
                ui.ctx()
                    .input()
                    .raw
                    .hovered_files
                    .last()
                    .unwrap()
                    .clone()
                    .path
                    .unwrap()
                    .file_name()
                    .unwrap(),
            );
        }

        let response = ui
            .add(
                TextEdit::multiline(&mut "")
                    .interactive(false)
                    .desired_rows(2)
                    .font(FontSelection::Style(TextStyle::Heading))
                    .hint_text(text),
            )
            .on_hover_cursor(CursorIcon::PointingHand)
            .interact(Sense::click());

        if response.clicked() {
            debug!("opening file dialog");
            if let Some(opened_path) = rfd::FileDialog::new()
                .add_filter("JSON files", &["json"])
                .pick_file()
            {
                file_path = opened_path.display().to_string();
            }
        }

        if !ui.ctx().input().raw.dropped_files.is_empty() {
            file_path = ui
                .ctx()
                .input()
                .raw
                .dropped_files
                .last()
                .unwrap()
                .path
                .clone()
                .unwrap()
                .display()
                .to_string();
        }

        self.update(file_path);
    }
}
