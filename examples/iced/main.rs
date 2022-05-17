mod network;
mod sources;

use chrono::offset::{Local, TimeZone};
use chrono::prelude::*;
use iced::{
    button, executor, Alignment, Application, Button, Column, Command, Element, Image, Settings,
    Text,
};
use plotters::prelude::*;
use sources::binance::client::{Client, Kline};
use sources::binance::interval::Interval;
use plotters_iced::{Chart, ChartWidget, DrawingBackend, ChartBuilder};

const HEIGHT: u32 = 768;
const WIDTH: u32 = 1024;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    LoadPressed,
    DataReceived(Vec<Kline>),
}

struct App {
    pair: String,
    values: Vec<Kline>,
    btn_load: button::State,
}

impl App {
    fn draw(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer: Vec<u8> = vec![0; 1024 * 768];
        let root = BitMapBackend::with_buffer(&mut buffer, (WIDTH, HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        if self.values.len() < 2 {
            return Ok(());
        }

        let (min_x, max_x) = (
            parse_time(self.values[0].t_close),
            parse_time(self.values.last().unwrap().t_close),
        );

        let (min_y, max_y) = (
            self.values.iter().min().unwrap().close,
            self.values.iter().max().unwrap().close,
        );

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .caption(format!("{}", self.pair), ("sans-serif", 32.0).into_font())
            .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

        chart.configure_mesh().light_line_style(&WHITE).draw()?;

        chart.draw_series(self.values.iter().map(|x| {
            let res = CandleStick::new(
                parse_time(x.t_close),
                x.open,
                x.high,
                x.low,
                x.close,
                GREEN.filled(),
                RED,
                15,
            );
            res
        }))?;

        // To avoid the IO failure being ignored silently, we manually call the present function
        root.present();

        Ok(())
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_f: ()) -> (Self, Command<Message>) {
        (
            Self {
                pair: "BTCUSDT".to_string(),
                values: vec![],
                btn_load: button::State::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("hedgegraph")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPressed => Command::perform(
                Client::kline(self.pair.clone(), Interval::Minute, 1651995344000, 100),
                Message::DataReceived,
            ),
            Message::DataReceived(data) => {
                self.values = data;
                self.draw().unwrap();
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .align_items(Alignment::Center)
            .push(
                Button::new(&mut self.btn_load, Text::new("load data"))
                    .on_press(Message::LoadPressed),
            )
            // .push(
            //     Image::new()
            // )
            .into()
    }
}

fn parse_time(t: i64) -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(t / 1000, 0), Utc)
}
