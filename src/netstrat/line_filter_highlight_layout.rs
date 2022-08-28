use eframe::epaint::text::TextWrapping;
use egui::{text::LayoutJob, TextFormat, Color32};

pub fn line_filter_highlight_layout(string: &str, filter: &String) -> LayoutJob {
    let mut job = LayoutJob {
        wrap: TextWrapping {
            break_anywhere: false,
            ..Default::default()
        },
        ..Default::default()
    };
    // need to work with 2 strings to preserve original register
    let mut text = string.to_string();
    let mut normalized_text = text.to_lowercase();
    while !text.is_empty() {
        let filter_offset_res = normalized_text.find(filter.to_lowercase().as_str());

        let mut drain_bound = text.len();
        if !filter.is_empty() {
            if let Some(filter_offset) = filter_offset_res {
                drain_bound = filter_offset + filter.len();

                let plain = &text.as_str()[..filter_offset];
                job.append(plain, 0.0, TextFormat::default());

                let highlighted = &text.as_str()[filter_offset..drain_bound];
                job.append(
                    highlighted,
                    0.0,
                    TextFormat {
                        background: Color32::YELLOW,
                        ..Default::default()
                    },
                );

                text.drain(..drain_bound);
                normalized_text.drain(..drain_bound);
                continue;
            }
        }

        let plain = &text.as_str()[..drain_bound];
        job.append(plain, 0.0, TextFormat::default());
        text.drain(..drain_bound);
    }
    job
}