//! État central de l'application (`DuckApp`) et assemblage des vues.

use crate::ai::AiPlanActions;
use crate::ics::{courses_for_date, fetch_all_courses, CourseEvent};
use crate::models::{AppConfig, RepeatMode, SaveData, TaskColor, TaskPriority};
use crate::theme::Palette;
use crate::ui::{ai_modal, header, tabs, timeline};
use chrono::{Datelike, Days, Local, NaiveDate};
use eframe::egui;
use std::fs;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone, Copy)]
pub enum MainTab {
    Chronologie,
    Taches,
}

/// État temporaire du formulaire de création par clic droit sur la chronologie
pub struct NewTaskDraft {
    pub open: bool,
    pub title: String,
    pub start_hour: u32,
    pub start_minute: u32,
    pub duration_minutes: u32,
    pub priority: TaskPriority,
    pub color: TaskColor,
    pub repeat_mode: RepeatMode,
    pub repeat_count: u32,
}

impl Default for NewTaskDraft {
    fn default() -> Self {
        Self {
            open: false,
            title: String::new(),
            start_hour: 12,
            start_minute: 0,
            duration_minutes: 60,
            priority: TaskPriority::Flexible,
            color: TaskColor::RedSport,
            repeat_mode: RepeatMode::Once,
            repeat_count: 1,
        }
    }
}

pub struct DuckApp {
    pub config: AppConfig,
    pub save: SaveData,
    pub all_courses: Vec<CourseEvent>,
    pub selected_date: NaiveDate,
    pub calendar_open: bool,
    pub calendar_month: NaiveDate,
    pub active_tab: MainTab,
    pub always_on_top: bool,
    pub timeline_zoom: f32,
    pub timer_running: bool,
    pub timer_remaining: u64,
    pub last_tick: Instant,
    pub new_task: NewTaskDraft,
    /// Stocke `(id_tache, titre_tache)` lorsqu'on clique gauche sur une activité à supprimer
    pub task_to_delete: Option<(String, String)>,
    /// État de la fenêtre Coach IA
    pub ai_modal_open: bool,
    pub ai_prompt: String,
    pub ai_loading: bool,
    pub ai_last_reply: String,
    pub ai_receiver: Option<Receiver<Result<AiPlanActions, String>>>,
}

impl DuckApp {
    pub fn new() -> Self {
        let config_str = fs::read_to_string("config.json")
            .expect("Impossible de lire config.json à la racine du projet !");
        let config: AppConfig =
            serde_json::from_str(&config_str).expect("Format JSON invalide dans config.json");

        let mut save = SaveData::load();
        let today = Local::now().date_naive();
        save.check_new_day(&today.format("%Y-%m-%d").to_string());

        let all_courses = fetch_all_courses(&config.ics_url);
        let weekday = today.weekday().to_string();
        let default_timer = config
            .sport_routine
            .get(&weekday)
            .map(|s| s.timer_seconds)
            .unwrap_or(0);

        let first_of_month =
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

        Self {
            config,
            save,
            all_courses,
            selected_date: today,
            calendar_open: false,
            calendar_month: first_of_month,
            active_tab: MainTab::Chronologie,
            always_on_top: true,
            timeline_zoom: 52.0,
            timer_running: false,
            timer_remaining: default_timer,
            last_tick: Instant::now(),
            new_task: NewTaskDraft::default(),
            task_to_delete: None,
            ai_modal_open: false,
            ai_prompt: String::new(),
            ai_loading: false,
            ai_last_reply: String::new(),
            ai_receiver: None,
        }
    }

    fn update_timer(&mut self) {
        if self.timer_running && self.last_tick.elapsed().as_secs() >= 1 {
            self.last_tick = Instant::now();
            if self.timer_remaining > 0 {
                self.timer_remaining -= 1;
            } else {
                self.timer_running = false;
            }
        }
    }

    /// Gère les raccourcis clavier (flèches gauche/droite pour changer de jour)
    fn handle_keyboard_navigation(&mut self, ctx: &egui::Context) {
        // Ne pas changer de jour si l'utilisateur écrit ou si une modale est ouverte
        if ctx.wants_keyboard_input()
            || self.new_task.open
            || self.task_to_delete.is_some()
            || self.ai_modal_open
        {
            return;
        }

        let (go_prev, go_next) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
            )
        });

        if go_prev {
            self.selected_date = self
                .selected_date
                .checked_sub_days(Days::new(1))
                .unwrap_or(self.selected_date);
        }
        if go_next {
            self.selected_date = self
                .selected_date
                .checked_add_days(Days::new(1))
                .unwrap_or(self.selected_date);
        }
    }

    /// Calcule les tâches réalisées et totales (cours + routines + tâches personnalisées du jour)
    fn daily_progress(
        &self,
        day_courses: &[CourseEvent],
        weekday_en: &str,
        is_today: bool,
    ) -> (u32, u32) {
        let mut total = day_courses.len() as u32;
        let mut done = day_courses
            .iter()
            .filter(|c| self.save.completed_courses.contains(&c.uid))
            .count() as u32;

        if self.config.hair_routine.contains_key(weekday_en) {
            total += 1;
            if is_today && self.save.hair_done_today {
                done += 1;
            }
        }
        if self.config.sport_routine.contains_key(weekday_en) {
            total += 1;
            if is_today && self.save.sport_done_today {
                done += 1;
            }
        }

        for task in &self.save.custom_tasks {
            if task.occurs_on(self.selected_date) {
                total += 1;
                let key = task.occurrence_key(self.selected_date);
                if self.save.completed_custom_occurrences.contains(&key) {
                    done += 1;
                }
            }
        }

        (done, total)
    }
}

impl eframe::App for DuckApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_timer();
        self.handle_keyboard_navigation(ctx);
        ctx.request_repaint_after(Duration::from_millis(100));

        let now = Local::now();
        let today = now.date_naive();
        self.save
            .check_new_day(&today.format("%Y-%m-%d").to_string());

        let day_courses = courses_for_date(&self.all_courses, self.selected_date);
        let weekday_en = self.selected_date.weekday().to_string();
        let is_today = self.selected_date == today;
        let (done_tasks, total_tasks) = self.daily_progress(&day_courses, &weekday_en, is_today);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Palette::BG_MAIN)
                    .inner_margin(18.0),
            )
            .show(ctx, |ui| {
                let avail_w = ui.available_width();

                header::render_top_bar(self, ctx, ui, today);

                if self.calendar_open {
                    ui.add_space(6.0);
                    header::render_calendar_popup(self, ui, today);
                }

                ui.add_space(8.0);
                header::render_date_selector(self, ui, is_today);
                ui.add_space(10.0);

                header::render_bento_stats(ui, avail_w, day_courses.len(), done_tasks, total_tasks);
                ui.add_space(10.0);

                let nav_bar_reserved = 68.0;
                let content_height = (ui.available_height() - nav_bar_reserved).max(200.0);
                let inner_box_w = avail_w - 22.0;

                match self.active_tab {
                    MainTab::Chronologie => timeline::render_timeline(
                        self,
                        ctx,
                        ui,
                        inner_box_w,
                        content_height,
                        &day_courses,
                        &weekday_en,
                        is_today,
                        now,
                    ),
                    MainTab::Taches => tabs::render_tasks_tab(
                        self,
                        ui,
                        inner_box_w,
                        content_height,
                        &day_courses,
                        &weekday_en,
                        is_today,
                    ),
                }

                ui.add_space(10.0);
                header::render_bottom_nav(self, ui);
            });

        // Fenêtre flottante du Coach IA
        ai_modal::render_ai_modal(self, ctx);
    }
}
