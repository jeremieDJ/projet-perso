//! Modèles de données : configuration externe (`config.json`),
//! tâches personnalisées avec priorités et sauvegarde locale (`save.json`).

use crate::theme::Palette;
use chrono::{Datelike, NaiveDate};
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HairDay {
    pub title: String,
    pub products: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportDay {
    pub title: String,
    pub details: String,
    pub timer_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ics_url: String,
    #[serde(default)]
    pub gemini_api_key: String,
    #[serde(default = "default_school_station")]
    pub school_station: String,
    #[serde(default = "default_home_station")]
    pub home_station: String,
    #[serde(default = "default_walk_minutes")]
    pub walk_to_station_minutes: u32,
    #[serde(default)]
    pub hair_routine: HashMap<String, HairDay>,
    #[serde(default)]
    pub sport_routine: HashMap<String, SportDay>,
}

fn default_school_station() -> String {
    "Sion, gare".to_string()
}
fn default_home_station() -> String {
    "Mollens VS".to_string()
}
fn default_walk_minutes() -> u32 {
    10
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskColor {
    RedSport,
    PurpleOther,
    GreenCare,
    GoldPerso,
    BlueStudy,
}

impl TaskColor {
    pub fn to_color32(self) -> Color32 {
        match self {
            TaskColor::RedSport => Palette::RED_SPORT,
            TaskColor::PurpleOther => Palette::PURPLE_REWARD,
            TaskColor::GreenCare => Palette::GREEN_ACCENT,
            TaskColor::GoldPerso => Palette::GOLD_ACCENT,
            TaskColor::BlueStudy => Palette::BLUE_COURSE,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TaskColor::RedSport => "Sport",
            TaskColor::PurpleOther => "Activité autre",
            TaskColor::GreenCare => "Soin",
            TaskColor::GoldPerso => "Perso",
            TaskColor::BlueStudy => "Études",
        }
    }
}

/// Niveau de priorité d'une activité (détermine ce que l'IA a le droit de déplacer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Niveau 1 : Sport, loisirs -> décalé en premier si un imprévu arrive
    Flexible = 1,
    /// Niveau 2 : Soins cheveux / routines -> déplacé seulement si conflit avec Haute/Fixe
    Medium = 2,
    /// Niveau 3 : Rendez-vous médical, impératif -> écrase ou décale les priorités 1 et 2
    High = 3,
    /// Niveau 4 : Inamovible (comme les cours) -> ne peut jamais être déplacé
    Fixed = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Flexible
    }
}

impl TaskPriority {
    pub fn badge(self) -> &'static str {
        match self {
            TaskPriority::Flexible => "P1",
            TaskPriority::Medium => "P2",
            TaskPriority::High => "P3!",
            TaskPriority::Fixed => "FIXE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Once,
    EveryWeek,
    EveryTwoWeeks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTask {
    pub id: String,
    pub title: String,
    pub start_date: String,
    pub start_hour: u32,
    pub start_minute: u32,
    pub duration_minutes: u32,
    pub color: TaskColor,
    #[serde(default)]
    pub priority: TaskPriority,
    pub repeat_mode: RepeatMode,
    pub repeat_count: u32,
    #[serde(default)]
    pub skipped_dates: Vec<String>,
}

impl CustomTask {
    pub fn occurs_on(&self, target: NaiveDate) -> bool {
        let target_str = target.format("%Y-%m-%d").to_string();
        if self.skipped_dates.contains(&target_str) {
            return false;
        }

        let Ok(start) = NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d") else {
            return false;
        };
        if target < start || target.weekday() != start.weekday() {
            return false;
        }
        let days_diff = (target - start).num_days();
        if days_diff % 7 != 0 {
            return false;
        }
        let weeks_diff = (days_diff / 7) as u32;

        match self.repeat_mode {
            RepeatMode::Once => weeks_diff == 0,
            RepeatMode::EveryWeek => weeks_diff < self.repeat_count.max(1),
            RepeatMode::EveryTwoWeeks => {
                weeks_diff % 2 == 0 && (weeks_diff / 2) < self.repeat_count.max(1)
            }
        }
    }

    pub fn occurrence_key(&self, date: NaiveDate) -> String {
        format!("{}_{}", self.id, date.format("%Y-%m-%d"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub xp: u32,
    pub level: u32,
    pub seeds: u32,
    pub last_date: String,
    pub hair_done_today: bool,
    pub sport_done_today: bool,
    pub completed_courses: Vec<String>,
    #[serde(default)]
    pub custom_tasks: Vec<CustomTask>,
    #[serde(default)]
    pub completed_custom_occurrences: Vec<String>,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            xp: 0,
            level: 1,
            seeds: 20,
            last_date: String::new(),
            hair_done_today: false,
            sport_done_today: false,
            completed_courses: Vec::new(),
            custom_tasks: Vec::new(),
            completed_custom_occurrences: Vec::new(),
        }
    }
}

impl SaveData {
    pub fn load() -> Self {
        fs::read_to_string("save.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write("save.json", json);
        }
    }

    pub fn check_new_day(&mut self, today_str: &str) {
        if self.last_date != today_str {
            self.last_date = today_str.to_string();
            self.hair_done_today = false;
            self.sport_done_today = false;
            self.completed_courses.clear();
            self.save();
        }
    }

    pub fn add_reward(&mut self, xp_gain: u32, seeds_gain: u32) {
        self.xp += xp_gain;
        self.seeds += seeds_gain;
        let xp_needed = self.level * 100;
        if self.xp >= xp_needed {
            self.xp -= xp_needed;
            self.level += 1;
            self.seeds += 25;
        }
        self.save();
    }

    pub fn remove_reward(&mut self, xp_loss: u32, seeds_loss: u32) {
        self.seeds = self.seeds.saturating_sub(seeds_loss);
        if self.xp >= xp_loss {
            self.xp -= xp_loss;
        } else if self.level > 1 {
            self.level -= 1;
            let prev_max = self.level * 100;
            self.xp = (prev_max + self.xp).saturating_sub(xp_loss);
            self.seeds = self.seeds.saturating_sub(25);
        } else {
            self.xp = 0;
        }
        self.save();
    }
}
