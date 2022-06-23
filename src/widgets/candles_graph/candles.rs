use std::rc::Rc;

use egui::{
    plot::{BoxElem, BoxPlot, BoxSpread, LinkedAxisGroup, Plot},
    Color32, Response, Stroke, Widget,
};

use super::data::Data;

#[derive(Clone)]
pub struct Candles {
    data: Data,
    val: Vec<BoxElem>,
    axes_group: LinkedAxisGroup,
}

impl Default for Candles {
    fn default() -> Self {
        Self {
            data: Default::default(),
            val: Default::default(),
            axes_group: LinkedAxisGroup::new(false, false),
        }
    }
}

impl Candles {
    pub fn new(data: Data, axes_group: LinkedAxisGroup) -> Self {
        let val: Vec<BoxElem> = data
            .vals
            .iter()
            .map(|k| -> BoxElem {
                BoxElem::new(
                    (k.t_open + k.t_close) as f64 / 2.0,
                    BoxSpread::new(
                        k.low as f64,
                        {
                            match k.open > k.close {
                                true => k.close as f64,
                                false => k.open as f64,
                            }
                        },
                        k.open as f64, // we don't need to see median for candle
                        {
                            match k.open > k.close {
                                true => k.open as f64,
                                false => k.close as f64,
                            }
                        },
                        k.high as f64,
                    ),
                )
                .name(Data::format_ts(k.t_close as f64))
                .stroke(Stroke::new(1.0, Data::k_color(k)))
                .fill(Data::k_color(k))
                .whisker_width(0.0)
                .box_width((k.t_open - k.t_close) as f64 * 0.9)
            })
            .collect();

        Self {
            data,
            val,
            axes_group,
        }
    }
}

impl Widget for &Candles {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        Plot::new("candles")
            .link_axis(self.axes_group.clone())
            .label_formatter(|_, v| -> String { format!("{}", Data::format_ts(v.x)) })
            .x_axis_formatter(|v, _range| Data::format_ts(v))
            .include_x(self.data.max_x())
            .include_y(self.data.max_y())
            .show(ui, |plot_ui| {
                plot_ui.box_plot(
                    BoxPlot::new(self.val.clone())
                        .element_formatter(Box::new(|el, p| -> String {
                            format!(
                                "open: {:.8}\nclose: {:.8}\nhigh: {:.8}\nlow: {:.8}\n{}",
                                {
                                    match el.fill == Color32::LIGHT_RED {
                                        true => el.spread.quartile3,
                                        false => el.spread.quartile1,
                                    }
                                },
                                {
                                    match el.fill == Color32::LIGHT_RED {
                                        true => el.spread.quartile1,
                                        false => el.spread.quartile3,
                                    }
                                },
                                el.spread.upper_whisker,
                                el.spread.lower_whisker,
                                Data::format_ts(el.argument),
                            )
                        }))
                        .vertical(),
                );
            })
            .response
    }
}
