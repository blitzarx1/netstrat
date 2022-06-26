use egui::{Response, Visuals, Widget};

static LIGHT_MODE_SYMBOL: &str = "🔆";
static DARK_MODE_SYMBOL: &str = "🌙";

pub struct Theme {
    dark_mode: bool,
}

impl Theme {
    pub fn new() -> Self {
        Self { dark_mode: true }
    }
}

impl Widget for &mut Theme {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        if self.dark_mode {
            ui.ctx().set_visuals(Visuals::dark())
        } else {
            ui.ctx().set_visuals(Visuals::light())
        }

        let btn = ui.button({
            match self.dark_mode {
                true => LIGHT_MODE_SYMBOL,
                false => DARK_MODE_SYMBOL,
            }
        });

        if btn.clicked() {
            self.dark_mode = !self.dark_mode
        };

        btn
    }
}
