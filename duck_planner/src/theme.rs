//! Palette de couleurs de l'interface et utilitaires de formatage de texte.

use chrono::{Datelike, NaiveDate};
use eframe::egui::Color32;

/// Regroupe toutes les couleurs du thème sombre chaud
pub struct Palette;

impl Palette {
    pub const BG_MAIN: Color32 = Color32::from_rgb(25, 24, 22);
    pub const BG_CARD: Color32 = Color32::from_rgb(35, 33, 30);
    pub const BORDER: Color32 = Color32::from_rgb(50, 47, 43);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(242, 239, 233);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 143, 136);
    pub const GREEN_ACCENT: Color32 = Color32::from_rgb(134, 184, 107);
    pub const GOLD_ACCENT: Color32 = Color32::from_rgb(217, 155, 56);
    pub const CORAL_NOW: Color32 = Color32::from_rgb(224, 109, 83);
    pub const RED_SPORT: Color32 = Color32::from_rgb(224, 82, 82);
    pub const BLUE_COURSE: Color32 = Color32::from_rgb(66, 135, 245);
    pub const PURPLE_REWARD: Color32 = Color32::from_rgb(160, 120, 220);
}

/// Formate une date en français abrégé (ex: "lun., 23 nov.")
pub fn french_date_label(date: NaiveDate) -> String {
    let day_name = match date.weekday() {
        chrono::Weekday::Mon => "lun.",
        chrono::Weekday::Tue => "mar.",
        chrono::Weekday::Wed => "mer.",
        chrono::Weekday::Thu => "jeu.",
        chrono::Weekday::Fri => "ven.",
        chrono::Weekday::Sat => "sam.",
        chrono::Weekday::Sun => "dim.",
    };
    let month_name = match date.month() {
        1 => "janv.",
        2 => "févr.",
        3 => "mars",
        4 => "avr.",
        5 => "mai",
        6 => "juin",
        7 => "juil.",
        8 => "août",
        9 => "sept.",
        10 => "oct.",
        11 => "nov.",
        _ => "déc.",
    };
    format!("{}, {} {}", day_name, date.day(), month_name)
}

/// Nettoie et raccourcit le libellé d'un cours pour qu'il tienne proprement dans l'interface
pub fn clean_course_label(summary: &str, location: &str) -> String {
    if summary == "Anglais" {
        return format!("{} {}", summary, location);
    }

    let short_sum = summary
        .split_once('•')
        .map(|(_, right)| right.trim())
        .unwrap_or(summary)
        .strip_prefix("HEI ")
        .unwrap_or(summary);

    let short_loc = location
        .split_whitespace()
        .next()
        .unwrap_or(location)
        .strip_prefix("VS-ENP.23.")
        .unwrap_or(location);

    if short_loc.is_empty() {
        short_sum.to_string()
    } else {
        format!("{} • {}", short_sum, short_loc)
    }
}
