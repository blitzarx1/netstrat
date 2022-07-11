use super::AppWindow;
use crossbeam::channel::{unbounded, Receiver};
use egui::{Response, Ui, Widget, Window};

pub struct TimeRangeChooser {
    symbol: String,
    symbol_chan: Receiver<String>,
    valid: bool,
    visible: bool,
}

impl TimeRangeChooser {
    pub fn new(visible: bool, r: Receiver<String>) -> Self {
        Self {
            symbol: String::new(),
            symbol_chan: r,
            valid: true,
            visible,
        }
    }
}

impl AppWindow for TimeRangeChooser {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("props").clicked() {
            self.visible = !self.visible
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let symbol_wrapped = self
            .symbol_chan
            .recv_timeout(std::time::Duration::from_millis(10));

        match symbol_wrapped {
            Ok(symbol) => {
                self.symbol = symbol;
            }
            Err(_) => {}
        }
        
        // TODO: make window always on top; this is not implemented in egui yet
        Window::new(self.symbol.to_string())
            .open(&mut self.visible)
            .drag_bounds(ui.max_rect())
            .resizable(false)
            .show(ui.ctx(), |ui| {

                // ui.collapsing("time period", |ui| {
                //     ui.horizontal_wrapped(|ui| {
                //         ui.add(
                //             egui_extras::DatePickerButton::new(&mut self.graph_props.date_start)
                //                 .id_source("datepicker_start"),
                //         );
                //         ui.label("date start");
                //     });
                //     ui.horizontal_wrapped(|ui| {
                //         ui.add(
                //             egui_extras::DatePickerButton::new(&mut self.graph_props.date_end)
                //                 .id_source("datepicker_end"),
                //         );
                //         ui.label("date end");
                //     });
                //     ui.horizontal_wrapped(|ui| {
                //         ui.add(&mut self.time_start);
                //         ui.label("time start");
                //     });
                //     ui.horizontal_wrapped(|ui| {
                //         ui.add(&mut self.time_end);
                //         ui.label("time end");
                //     });
                // });
                // ui.collapsing("interval", |ui| {
                //     egui::ComboBox::from_label("pick data interval")
                //         .selected_text(format!("{:?}", self.graph_props.interval))
                //         .show_ui(ui, |ui| {
                //             ui.selectable_value(
                //                 &mut self.graph_props.interval,
                //                 Interval::Day,
                //                 "Day",
                //             );
                //             ui.selectable_value(
                //                 &mut self.graph_props.interval,
                //                 Interval::Hour,
                //                 "Hour",
                //             );
                //             ui.selectable_value(
                //                 &mut self.graph_props.interval,
                //                 Interval::Minute,
                //                 "Minute",
                //             );
                //         });
                // });

                // ui.add_space(5f32);

                // ui.horizontal(|ui| {
                //     if ui.button("show").clicked() {
                //         let time_start = self.time_start.get_time();
                //         match time_start {
                //             Some(time) => {
                //                 self.valid = true;
                //                 self.graph_loading_state =
                //                     LoadingState::from_graph_props(&self.graph_props);
                //                 self.graph_loading_state.triggered = true;

                //                 self.graph_props.time_start = time;
                //                 let start =
                //                     self.graph_props.start_time().timestamp_millis().clone();
                //                 let pair = self.symbol.to_string();
                //                 let interval = self.graph_props.interval.clone();
                //                 let mut limit = self.graph_props.limit.clone();

                //                 if self.graph_loading_state.is_last_page() {
                //                     limit = self.graph_loading_state.last_page_limit
                //                 }

                //                 self.klines_promise = Some(Promise::spawn_async(async move {
                //                     Client::kline(pair, interval, start, limit).await
                //                 }));
                //             }
                //             None => {
                //                 self.valid = false;
                //             }
                //         }
                //     }

                //     if !self.valid {
                //         ui.label("invalid time range");
                //     }

                //     if ui.button("export").clicked() {
                //         let name = format!(
                //             "{}-{}-{}-{:?}",
                //             self.symbol,
                //             self.graph_props.start_time(),
                //             self.graph_props.end_time(),
                //             self.graph_props.interval,
                //         );
                //         let f = File::create(format!("{}.csv", name)).unwrap();

                //         let mut wtr = csv::Writer::from_writer(f);
                //         self.klines.iter().for_each(|el| {
                //             wtr.serialize(el).unwrap();
                //         });
                //         wtr.flush().unwrap();
                //     };
                // });
            });
    }
}
