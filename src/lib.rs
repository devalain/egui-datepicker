//! egui-datepicker adds a simple date picker widget.
//! Checkout the [example][ex]
//!
//!
//! ```no_run
//! use eframe::egui::Ui;
//! use chrono::prelude::*;
//! use std::fmt::Display;
//! use egui_datepicker::DatePicker;
//!
//! struct App {
//!     date: chrono::NaiveDate
//! }
//! impl App {
//!     fn draw_datepicker(&mut self, ui: &mut Ui) {
//!         ui.add(DatePicker::new("super_unique_id", &mut self.date));
//!     }
//! }
//! ```
//!
//! [ex]: ./examples/simple.rs

use std::hash::Hash;

pub use chrono::{
    offset::{FixedOffset, Local, Utc},
    NaiveDate,
};
use chrono::{prelude::*, Duration};
use eframe::{
    egui,
    egui::{Area, Color32, DragValue, Frame, Id, Key, Order, Response, RichText, Ui, Widget},
};
use num_traits::FromPrimitive;

/// Default values of fields are:
/// - sunday_first: `false`
/// - movable: `false`
/// - format_string: `"%Y-%m-%d"`
/// - weekend_func: `date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun`
pub struct DatePicker<'a> {
    id: Id,
    date: &'a mut NaiveDate,
    sunday_first: bool,
    movable: bool,
    format_string: String,
    weekend_color: Color32,
    weekend_func: fn(&NaiveDate) -> bool,
    highlight_weekend: bool,
}

impl<'a> DatePicker<'a> {
    /// Create new date picker with unique id and mutable reference to date.
    pub fn new<T: Hash>(id: T, date: &'a mut NaiveDate) -> Self {
        Self {
            id: Id::new(id),
            date,
            sunday_first: false,
            movable: false,
            format_string: String::from("%Y-%m-%d"),
            weekend_color: Color32::from_rgb(196, 0, 0),
            weekend_func: |date| date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun,
            highlight_weekend: true,
        }
    }

    /// If flag is set to true then first day in calendar will be sunday otherwise monday.
    /// Default is false
    #[must_use]
    pub fn sunday_first(mut self, flag: bool) -> Self {
        self.sunday_first = flag;
        self
    }

    /// If flag is set to true then date picker popup will be movable.
    /// Default is false
    #[must_use]
    pub fn movable(mut self, flag: bool) -> Self {
        self.movable = flag;
        self
    }

    ///Set date format.
    ///See the [chrono::format::strftime](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html) for the specification.
    #[must_use]
    pub fn date_format(mut self, new_format: impl ToString) -> Self {
        self.format_string = new_format.to_string();
        self
    }

    ///If highlight is true then weekends text color will be `weekend_color` instead default text
    ///color.
    #[must_use]
    pub fn highlight_weekend(mut self, highlight: bool) -> Self {
        self.highlight_weekend = highlight;
        self
    }

    ///Set weekends highlighting color.
    #[must_use]
    pub fn highlight_weekend_color(mut self, color: Color32) -> Self {
        self.weekend_color = color;
        self
    }

    /// Set function, which will decide if date is a weekend day or not.
    pub fn weekend_days(mut self, is_weekend: fn(&NaiveDate) -> bool) -> Self {
        self.weekend_func = is_weekend;
        self
    }

    /// Draw names of week days as 7 columns of grid without calling `Ui::end_row`
    fn show_grid_header(&mut self, ui: &mut Ui) {
        let day_indexes = if self.sunday_first {
            [6, 0, 1, 2, 3, 4, 5]
        } else {
            [0, 1, 2, 3, 4, 5, 6]
        };
        for i in day_indexes {
            let b = Weekday::from_u8(i).unwrap();
            ui.label(b.to_string());
        }
    }

    /// Get number of days between first day of the month and Monday ( or Sunday if field
    /// `sunday_first` is set to `true` )
    fn get_start_offset_of_calendar(&self, first_day: &NaiveDate) -> u32 {
        if self.sunday_first {
            first_day.weekday().num_days_from_sunday()
        } else {
            first_day.weekday().num_days_from_monday()
        }
    }

    /// Get number of days between first day of the next month and Monday ( or Sunday if field
    /// `sunday_first` is set to `true` )
    fn get_end_offset_of_calendar(&self, first_day: &NaiveDate) -> u32 {
        if self.sunday_first {
            (7 - (first_day).weekday().num_days_from_sunday()) % 7
        } else {
            (7 - (first_day).weekday().num_days_from_monday()) % 7
        }
    }

    fn show_calendar_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("calendar").show(ui, |ui| {
            self.show_grid_header(ui);
            let first_day_of_current_month = self.date.with_day(1).unwrap();
            let start_offset = self.get_start_offset_of_calendar(&first_day_of_current_month);
            let days_in_month = get_days_from_month(self.date.year(), self.date.month());
            let first_day_of_next_month =
                first_day_of_current_month + Duration::days(days_in_month);
            let end_offset = self.get_end_offset_of_calendar(&first_day_of_next_month);
            let start_date = first_day_of_current_month - Duration::days(start_offset.into());
            for i in 0..(start_offset as i64 + days_in_month + end_offset as i64) {
                if i % 7 == 0 {
                    ui.end_row();
                }
                let d = start_date + Duration::days(i);
                self.show_day_button(d, ui);
            }
        });
    }

    fn show_day_button(&mut self, date: NaiveDate, ui: &mut Ui) {
        ui.add_enabled_ui(self.date != &date, |ui| {
            ui.centered_and_justified(|ui| {
                if self.date.month() != date.month() {
                    ui.style_mut().visuals.button_frame = false;
                }
                if self.highlight_weekend && (self.weekend_func)(&date) {
                    ui.style_mut().visuals.override_text_color = Some(self.weekend_color);
                }
                if ui.button(date.day().to_string()).clicked() {
                    *self.date = date;
                }
            });
        });
    }

    /// Draw current month and buttons for next and previous month.
    fn show_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.show_month_control(ui);
            self.show_year_control(ui);
            if ui.button("Today").clicked() {
                *self.date = Local::now().date_naive();
            }
        });
    }

    fn date_step_year_button(&mut self, ui: &mut Ui, text: impl ToString, years: i32) {
        if ui.button(text.to_string()).clicked() {
            let year = self.date.year() + years;
            *self.date = self.date.with_year(year).unwrap();
        }
    }

    fn date_step_month_button(&mut self, ui: &mut Ui, text: impl ToString, months: i32) {
        if ui.button(text.to_string()).clicked() {
            let mut month0 = self.date.month0() as i32 + months;
            let mut year = self.date.year();
            if month0 < 0 {
                let dy = if month0 % 12 == 0 {
                    -month0 / 12
                } else {
                    -month0 / 12 + 1
                };
                year -= dy;
                month0 += 12 * dy;
            }
            year += month0 / 12;
            let month = (month0 % 12 + 1) as u32;
            *self.date = self.date.with_year(year).unwrap().with_month(month).unwrap();
        }
    }

    /// Draw drag value widget with current year and two buttons which substract and add 365 days
    /// to current date.
    fn show_year_control(&mut self, ui: &mut Ui) {
        self.date_step_year_button(ui, "<", -1);
        let mut drag_year = self.date.year();
        ui.add(DragValue::new(&mut drag_year));
        if drag_year != self.date.year() {
            *self.date = self.date.with_year(drag_year).unwrap();
        }
        self.date_step_year_button(ui, ">", 1);
    }

    /// Draw label(will be combobox in future) with current month and two buttons which substract and add 30 days
    /// to current date.
    fn show_month_control(&mut self, ui: &mut Ui) {
        self.date_step_month_button(ui, "<", -1);
        let month_string = chrono::Month::from_u32(self.date.month()).unwrap().name();
        // TODO: When https://github.com/emilk/egui/pull/543 is merged try to change label to combo box.
        ui.add(egui::Label::new(
            RichText::new(format!("{: <9}", month_string)).text_style(egui::TextStyle::Monospace),
        ));
        // let mut selected = self.date.month0() as usize;
        // egui::ComboBox::from_id_source(self.id.with("month_combo_box"))
        //     .selected_text(selected)
        //     .show_index(ui, &mut selected, 12, |i| {
        //         chrono::Month::from_usize(i + 1).unwrap().name().to_string()
        //     });
        // if selected != self.date.month0() as usize {
        //     *self.date = self.date.with_month0(selected as u32).unwrap();
        // }
        self.date_step_month_button(ui, ">", 1);
    }
}

impl<'a> Widget for DatePicker<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let formated_date = self.date.format(&self.format_string);
        let button_response = ui.button(formated_date.to_string());
        if button_response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(self.id));
        }

        if ui.memory(|mem| mem.is_popup_open(self.id)) {
            let mut area = Area::new(self.id)
                .order(Order::Foreground)
                .default_pos(button_response.rect.left_bottom());
            if !self.movable {
                area = area.movable(false);
            }
            let area_response = area
                .show(ui.ctx(), |ui| {
                    Frame::popup(ui.style()).show(ui, |ui| {
                        self.show_header(ui);
                        self.show_calendar_grid(ui);
                    });
                })
                .response;

            if !button_response.clicked()
                && (ui.input(|i| i.key_pressed(Key::Escape)) || area_response.clicked_elsewhere())
            {
                ui.memory_mut(|mem| mem.toggle_popup(self.id));
            }
        }
        button_response
    }
}

// https://stackoverflow.com/a/58188385
fn get_days_from_month(year: i32, month: u32) -> i64 {
    NaiveDate::from_ymd_opt(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days()
}
