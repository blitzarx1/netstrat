use std::fmt::Display;

use chrono::{NaiveTime, Utc};
use egui::widgets::{TextEdit, Widget};
use egui::Color32;
use tracing::info;

/// Time hold value for hours, minutes and seconds validating them.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Time {
    hours: u32,
    minutes: u32,
    seconds: u32,
}

impl Time {
    pub fn new(hours: u32, minutes: u32, seconds: u32) -> Option<Self> {
        let t = Self {
            hours,
            minutes,
            seconds,
        };

        if !t.valid() {
            return None;
        }

        Some(t)
    }

    pub fn valid(&self) -> bool {
        self.hours < 24 && self.minutes < 60 && self.seconds < 60
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}:{}:{}", self.hours, self.minutes, self.seconds).as_str())
    }
}

// TimeInput is a widget that allows the user to enter a time.
#[derive(Default)]
pub struct TimeInput {
    time: Time,
    val: String,
    valid: bool,
}

impl TimeInput {
    pub fn new(hours: u32, minutes: u32, seconds: u32) -> Self {
        let t = Time::new(hours, minutes, seconds);
        match t {
            Some(time) => Self {
                time,
                val: format!("{}", time),
                valid: true,
            },
            None => Default::default(),
        }
    }

    /// Returns chrono::NaiveTime from the time input.
    pub fn get_time(&self) -> Option<NaiveTime> {
        if !self.valid {
            info!("faield to parse time from val: {}", self.val);
            return None;
        }

        let time = self.time;
        info!("parsed time: {time}");
        Some(NaiveTime::from_hms(time.hours, time.minutes, time.seconds))
    }

    fn parse_val(&mut self) -> Option<Time> {
        let mut split = self.val.split(':');
        let hours = split.next()?.parse::<u32>().ok()?;
        let minutes = split.next()?.parse::<u32>().ok()?;
        let seconds = split.next()?.parse::<u32>().ok()?;

        if split.next().is_some() {
            return None;
        }

        Some(Time::new(hours, minutes, seconds)?)
    }
}

impl Widget for &mut TimeInput {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let t = self.parse_val();
        match t {
            Some(time) => {
                self.time = time;
                self.valid = time.valid();
            }
            None => {
                self.valid = false;
            }
        }

        ui.horizontal_wrapped(|ui| {
            let mut w = TextEdit::singleline(&mut self.val)
                .desired_width(100.0)
                .hint_text("hh:mm:ss 24h");
            if !self.valid {
                w = w.text_color(Color32::LIGHT_RED);
            }

            ui.add(w);
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_time_new() {
        let t_valid = Time::new(23, 24, 24);
        let t_invalid = Time::new(32, 32, 32);
        let t_invalid_corner = Time::new(24, 24, 24);

        assert_eq!(t_valid.is_some(), true);
        assert_eq!(t_invalid, None);
        assert_eq!(t_invalid_corner, None);
    }

    #[test]
    fn test_time_input_parse_val() {
        let mut ti = TimeInput {
            time: Time::default(),
            val: "23:23:23".to_string(),
            valid: false,
        };

        let t = ti.parse_val();

        assert_eq!(t, Time::new(23, 23, 23));
    }
}
