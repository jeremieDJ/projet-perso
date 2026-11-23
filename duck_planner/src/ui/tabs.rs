//! Contenu de l'onglet "Tâches" (cases à cocher bidirectionnelles + chronomètre de sport).

use crate::app::DuckApp;
use crate::ics::CourseEvent;
use crate::theme::{clean_course_label, Palette};
use eframe::egui;
use std::time::Instant;

pub fn render_tasks_tab(
    app: &mut DuckApp,
    ui: &mut egui::Ui,
    inner_box_w: f32,
    content_height: f32,
    day_courses: &[CourseEvent],
    weekday_en: &str,
    is_today: bool,
) {
    egui::Frame::none()
        .fill(Palette::BG_CARD)
        .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
        .rounding(12.0)
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.set_height(content_height);
            ui.set_width(inner_box_w - 8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // --- 1. TÂCHES PERSONNALISÉES DU JOUR (CRÉÉES PAR CLIC DROIT) ---
                let mut tasks_today_indices = Vec::new();
                for (idx, t) in app.save.custom_tasks.iter().enumerate() {
                    if t.occurs_on(app.selected_date) {
                        tasks_today_indices.push(idx);
                    }
                }

                if !tasks_today_indices.is_empty() {
                    ui.label(
                        egui::RichText::new("✨ Activités planifiées")
                            .strong()
                            .color(Palette::PURPLE_REWARD),
                    );
                    let mut id_to_delete: Option<String> = None;

                    for idx in tasks_today_indices {
                        let task = app.save.custom_tasks[idx].clone();
                        let occ_key = task.occurrence_key(app.selected_date);
                        let done = app.save.completed_custom_occurrences.contains(&occ_key);
                        let mut checked = done;

                        ui.horizontal(|ui| {
                            let label = format!(
                                "{:02}:{:02} ({}m) | {}",
                                task.start_hour,
                                task.start_minute,
                                task.duration_minutes,
                                task.title
                            );
                            if ui
                                .checkbox(
                                    &mut checked,
                                    egui::RichText::new(label).color(task.color.to_color32()),
                                )
                                .changed()
                            {
                                if checked && !done {
                                    app.save.completed_custom_occurrences.push(occ_key);
                                    app.save.add_reward(25, 10);
                                } else if !checked && done {
                                    app.save
                                        .completed_custom_occurrences
                                        .retain(|k| k != &occ_key);
                                    app.save.remove_reward(25, 10);
                                }
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("🗑")
                                        .on_hover_text("Supprimer cette activité récurrente")
                                        .clicked()
                                    {
                                        id_to_delete = Some(task.id.clone());
                                    }
                                },
                            );
                        });
                    }

                    if let Some(del_id) = id_to_delete {
                        app.save.custom_tasks.retain(|t| t.id != del_id);
                        app.save
                            .completed_custom_occurrences
                            .retain(|k| !k.starts_with(&del_id));
                        app.save.save();
                    }

                    ui.separator();
                }

                // --- 2. SOIN CHEVEUX ---
                if let Some(hair) = app.config.hair_routine.get(weekday_en) {
                    ui.label(
                        egui::RichText::new("🧴 Soin cheveux")
                            .strong()
                            .color(Palette::GREEN_ACCENT),
                    );
                    let mut checked = is_today && app.save.hair_done_today;
                    if ui
                        .add_enabled(is_today, egui::Checkbox::new(&mut checked, &hair.title))
                        .changed()
                    {
                        if checked && !app.save.hair_done_today {
                            app.save.hair_done_today = true;
                            app.save.add_reward(20, 10);
                        } else if !checked && app.save.hair_done_today {
                            app.save.hair_done_today = false;
                            app.save.remove_reward(20, 10);
                        }
                    }
                    ui.label(
                        egui::RichText::new(&hair.products)
                            .size(12.0)
                            .color(Palette::TEXT_MUTED),
                    );
                    ui.separator();
                }

                // --- 3. SÉANCE DE SPORT + CHRONOMÈTRE ---
                if let Some(sport) = app.config.sport_routine.get(weekday_en) {
                    ui.label(
                        egui::RichText::new("🏋️ Séance de sport")
                            .strong()
                            .color(Palette::GOLD_ACCENT),
                    );
                    let mut checked = is_today && app.save.sport_done_today;
                    if ui
                        .add_enabled(is_today, egui::Checkbox::new(&mut checked, &sport.title))
                        .changed()
                    {
                        if checked && !app.save.sport_done_today {
                            app.save.sport_done_today = true;
                            app.save.add_reward(50, 25);
                        } else if !checked && app.save.sport_done_today {
                            app.save.sport_done_today = false;
                            app.save.remove_reward(50, 25);
                        }
                    }
                    ui.label(
                        egui::RichText::new(&sport.details)
                            .size(12.0)
                            .color(Palette::TEXT_MUTED),
                    );

                    if sport.timer_seconds > 0 && is_today {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            let mins = app.timer_remaining / 60;
                            let secs = app.timer_remaining % 60;
                            ui.monospace(format!("⏱ {:02}:{:02}", mins, secs));

                            let btn_txt = if app.timer_running { "Pause" } else { "Start" };
                            if ui.small_button(btn_txt).clicked() {
                                app.timer_running = !app.timer_running;
                                app.last_tick = Instant::now();
                            }
                            if ui.small_button("Reset").clicked() {
                                app.timer_running = false;
                                app.timer_remaining = sport.timer_seconds;
                            }
                        });
                    }
                    ui.separator();
                }

                // --- 4. COURS DU JOUR ---
                ui.label(
                    egui::RichText::new("📚 Cours du jour")
                        .strong()
                        .color(Palette::BLUE_COURSE),
                );
                if day_courses.is_empty() {
                    ui.label(
                        egui::RichText::new("Aucun cours")
                            .size(12.0)
                            .color(Palette::TEXT_MUTED),
                    );
                } else {
                    for course in day_courses {
                        let done = app.save.completed_courses.contains(&course.uid);
                        let mut checked = done;
                        let label = format!(
                            "{}-{} | {}",
                            course.start.format("%H:%M"),
                            course.end.format("%H:%M"),
                            clean_course_label(&course.summary, &course.location)
                        );
                        if ui.checkbox(&mut checked, label).changed() {
                            if checked && !done {
                                app.save.completed_courses.push(course.uid.clone());
                                app.save.add_reward(30, 15);
                            } else if !checked && done {
                                app.save.completed_courses.retain(|id| id != &course.uid);
                                app.save.remove_reward(30, 15);
                            }
                        }
                    }
                }
            });
        });
}
