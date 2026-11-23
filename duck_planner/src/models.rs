//! Modèles de données : configuration externe (`config.json`),
//! tâches personnalisées récurrentes et sauvegarde locale (`save.json`).

use crate::theme::Palette;
use chrono::{Datelike, NaiveDate};
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Routine capillaire pour un jour donné de la semaine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HairDay {
    pub title: String,
    pub products: String,
}

/// Séance de sport pour un jour donné de la semaine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportDay {
    pub title: String,
    pub details: String,
    pub timer_seconds: u64,
}

/// Configuration globale chargée depuis `config.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ics_url: String,
    pub hair_routine: HashMap<String, HairDay>,
    pub sport_routine: HashMap<String, SportDay>,
}

/// Catégorie / couleur d'une tâche personnalisée
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

/// Type de répétition hebdomadaire
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Once,
    EveryWeek,
    EveryTwoWeeks,
}

impl RepeatMode {
    pub fn label(self) -> &'static str {
        match self {
            RepeatMode::Once => "Une seule fois",
            RepeatMode::EveryWeek => "Chaque semaine (même jour)",
            RepeatMode::EveryTwoWeeks => "1 semaine sur 2 (même jour)",
        }
    }
}

// Tâche personnalisée créée par clic droit sur l'emploi du temps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTask {
    pub id: String,
    pub title: String,
    /// Date de première occurrence au format "YYYY-MM-DD"
    pub start_date: String,
    pub start_hour: u32,
    pub start_minute: u32,
    pub duration_minutes: u32,
    pub color: TaskColor,
    pub repeat_mode: RepeatMode,
    /// Nombre total d'occurrences (ex: 1 = une fois, 4 = 4 répétitions)
    pub repeat_count: u32,
    /// Dates spécifiques où cette occurrence a été supprimée ("YYYY-MM-DD")
    #[serde(default)]
    pub skipped_dates: Vec<String>,
}

impl CustomTask {
    /// Vérifie si cette tâche tombe à la date `target` selon sa règle de récurrence
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

    /// Identifiant unique d'une occurrence précise (ex: "custom_12345_2026-11-23")
    pub fn occurrence_key(&self, date: NaiveDate) -> String {
        format!("{}_{}", self.id, date.format("%Y-%m-%d"))
    }
}

/// Données de progression et tâches sauvegardées dans `save.json`
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
