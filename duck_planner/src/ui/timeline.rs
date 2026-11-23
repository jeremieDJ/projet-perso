//! Vue "Chronologie" : affiche l'emploi du temps vertical de 07h00 à 22h00,
//! permet d'ajouter une tâche par clic droit et de la supprimer par clic gauche.

use crate::app::DuckApp;
use crate::ics::CourseEvent;
use crate::models::{CustomTask, RepeatMode, TaskColor};
use crate::theme::{clean_course_label, Palette};
use chrono::{DateTime, Local, Timelike};
use eframe::egui;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(clippy::too_many_arguments)]
pub fn render_timeline(
    app: &mut DuckApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    inner_box_w: f32,
    content_height: f32,
    day_courses: &[CourseEvent],
    weekday_en: &str,
    is_today: bool,
    now: DateTime<Local>,
) {
    egui::Frame::none()
        .fill(Palette::BG_CARD)
        .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
        .rounding(12.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_height(content_height);
            ui.set_width(inner_box_w);

            // En-tête avec indication des clics + boutons zoom +/-
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Clic droit : ajouter  |  Clic gauche : supprimer")
                        .size(11.0)
                        .color(Palette::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("+").clicked() {
                        app.timeline_zoom = (app.timeline_zoom + 8.0).min(90.0);
                    }
                    if ui.small_button("-").clicked() {
                        app.timeline_zoom = (app.timeline_zoom - 8.0).max(34.0);
                    }
                });
            });

            egui::ScrollArea::vertical()
                .max_height(content_height - 32.0)
                .show(ui, |ui| {
                    let start_hour = 7;
                    let end_hour = 22;
                    let total_hours = (end_hour - start_hour) as f32;
                    let canvas_height = total_hours * app.timeline_zoom;

                    let (tl_rect, response) = ui.allocate_exact_size(
                        egui::vec2(inner_box_w - 14.0, canvas_height),
                        egui::Sense::click(),
                    );

                    let left_axis_w = 50.0;
                    let track_left = tl_rect.left() + left_axis_w;
                    let track_right = tl_rect.right() - 6.0;

                    // Pré-calcul des rectangles des tâches personnalisées pour détecter le clic gauche
                    let mut custom_hitboxes: Vec<(egui::Rect, String, String)> = Vec::new();
                    for task in &app.save.custom_tasks {
                        if task.occurs_on(app.selected_date) {
                            let s_h = task.start_hour as f32 + task.start_minute as f32 / 60.0;
                            let dur_h = task.duration_minutes as f32 / 60.0;
                            let e_h = (s_h + dur_h).min(end_hour as f32);

                            if e_h > start_hour as f32 && s_h < end_hour as f32 {
                                let y1 = tl_rect.top()
                                    + (s_h.max(start_hour as f32) - start_hour as f32)
                                        * app.timeline_zoom;
                                let y2 =
                                    tl_rect.top() + (e_h - start_hour as f32) * app.timeline_zoom;
                                let rect = egui::Rect::from_min_max(
                                    egui::pos2(track_left + 4.0, y1),
                                    egui::pos2(track_right - 4.0, y2.max(y1 + 22.0)),
                                );
                                custom_hitboxes.push((rect, task.id.clone(), task.title.clone()));
                            }
                        }
                    }

                    // --- CLIC GAUCHE SUR UNE ACTIVITÉ PERSONNALISÉE -> SUPPRIMER ---
                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            for (rect, id, title) in &custom_hitboxes {
                                if rect.contains(pos) {
                                    app.task_to_delete = Some((id.clone(), title.clone()));
                                    break;
                                }
                            }
                        }
                    }

                    // --- CLIC DROIT SUR LA GRILLE -> CRÉER UNE ACTIVITÉ ---
                    if response.secondary_clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let rel_y = (pos.y - tl_rect.top()).max(0.0);
                            let exact_hour = start_hour as f32 + (rel_y / app.timeline_zoom);
                            let h = (exact_hour.floor() as u32).clamp(7, 21);
                            let raw_min = ((exact_hour.fract()) * 60.0) as u32;
                            let m = ((raw_min + 7) / 15 * 15) % 60;

                            app.new_task.open = true;
                            app.new_task.title.clear();
                            app.new_task.start_hour = h;
                            app.new_task.start_minute = m;
                            app.new_task.duration_minutes = 60;
                            app.new_task.repeat_mode = RepeatMode::Once;
                            app.new_task.repeat_count = 1;
                        }
                    }

                    let p = ui.painter();

                    // 1. Repères horaires (07:00 à 22:00)
                    for h in start_hour..=end_hour {
                        let y = tl_rect.top() + (h - start_hour) as f32 * app.timeline_zoom;
                        p.text(
                            egui::pos2(tl_rect.left() + 4.0, y),
                            egui::Align2::LEFT_CENTER,
                            format!("{:02}:00", h),
                            egui::FontId::monospace(11.0),
                            Palette::TEXT_MUTED,
                        );
                        p.line_segment(
                            [egui::pos2(track_left, y), egui::pos2(track_right, y)],
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(44, 41, 37)),
                        );
                    }

                    // 2. Bloc Soin cheveux
                    if let Some(hair) = app.config.hair_routine.get(weekday_en) {
                        let y1 = tl_rect.top() + 0.20 * app.timeline_zoom;
                        let y2 = tl_rect.top() + 0.85 * app.timeline_zoom;
                        draw_timeline_block(
                            p,
                            track_left,
                            track_right,
                            y1,
                            y2,
                            Palette::GREEN_ACCENT,
                            &format!("🧴 {}", hair.title),
                        );
                    }

                    // 3. Blocs Cours (.ics)
                    for course in day_courses {
                        let s_h = course.start.hour() as f32 + course.start.minute() as f32 / 60.0;
                        let e_h = course.end.hour() as f32 + course.end.minute() as f32 / 60.0;
                        if e_h > start_hour as f32 {
                            let y1 = tl_rect.top() + (s_h - start_hour as f32) * app.timeline_zoom;
                            let y2 = tl_rect.top() + (e_h - start_hour as f32) * app.timeline_zoom;
                            let label = clean_course_label(&course.summary, &course.location);
                            draw_timeline_block(
                                p,
                                track_left,
                                track_right,
                                y1,
                                y2.max(y1 + 24.0),
                                Palette::BLUE_COURSE,
                                &label,
                            );
                        }
                    }

                    // 4. Bloc Sport config.json
                    if let Some(sport) = app.config.sport_routine.get(weekday_en) {
                        if sport.timer_seconds > 0 {
                            let y1 = tl_rect.top() + (18.5 - start_hour as f32) * app.timeline_zoom;
                            let y2 = y1 + 0.9 * app.timeline_zoom;
                            draw_timeline_block(
                                p,
                                track_left,
                                track_right,
                                y1,
                                y2,
                                Palette::GOLD_ACCENT,
                                &format!("🏋️ {}", sport.title),
                            );
                        }
                    }

                    // 5. Blocs des tâches personnalisées
                    for task in &app.save.custom_tasks {
                        if task.occurs_on(app.selected_date) {
                            let s_h = task.start_hour as f32 + task.start_minute as f32 / 60.0;
                            let dur_h = task.duration_minutes as f32 / 60.0;
                            let e_h = (s_h + dur_h).min(end_hour as f32);

                            if e_h > start_hour as f32 && s_h < end_hour as f32 {
                                let y1 = tl_rect.top()
                                    + (s_h.max(start_hour as f32) - start_hour as f32)
                                        * app.timeline_zoom;
                                let y2 =
                                    tl_rect.top() + (e_h - start_hour as f32) * app.timeline_zoom;
                                let lbl = format!(
                                    "{:02}:{:02} • {} ({} min)  🗑",
                                    task.start_hour,
                                    task.start_minute,
                                    task.title,
                                    task.duration_minutes
                                );
                                draw_timeline_block(
                                    p,
                                    track_left,
                                    track_right,
                                    y1,
                                    y2.max(y1 + 22.0),
                                    task.color.to_color32(),
                                    &lbl,
                                );
                            }
                        }
                    }

                    // 6. Ligne de l'heure actuelle
                    if is_today {
                        let cur_h = now.hour() as f32 + now.minute() as f32 / 60.0;
                        if cur_h >= start_hour as f32 && cur_h <= end_hour as f32 {
                            let cur_y =
                                tl_rect.top() + (cur_h - start_hour as f32) * app.timeline_zoom;
                            p.line_segment(
                                [
                                    egui::pos2(track_left, cur_y),
                                    egui::pos2(track_right - 44.0, cur_y),
                                ],
                                egui::Stroke::new(1.8_f32, Palette::CORAL_NOW),
                            );
                            let badge_r = egui::Rect::from_center_size(
                                egui::pos2(track_right - 21.0, cur_y),
                                egui::vec2(44.0, 17.0),
                            );
                            p.rect_filled(badge_r, 8.0, Palette::CORAL_NOW);
                            p.text(
                                badge_r.center(),
                                egui::Align2::CENTER_CENTER,
                                now.format("%H:%M").to_string(),
                                egui::FontId::monospace(10.5),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                });
        });

    render_new_task_modal(app, ctx);
    render_delete_task_modal(app, ctx);
}

/// Fenêtre de confirmation de suppression lors d'un clic gauche sur une activité
fn render_delete_task_modal(app: &mut DuckApp, ctx: &egui::Context) {
    let Some((task_id, task_title)) = app.task_to_delete.clone() else {
        return;
    };

    let is_recurring = app
        .save
        .custom_tasks
        .iter()
        .find(|t| t.id == task_id)
        .map(|t| t.repeat_mode != RepeatMode::Once)
        .unwrap_or(false);

    let mut open = true;
    egui::Window::new("🗑 Supprimer l'activité")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Palette::BG_CARD)
                .stroke(egui::Stroke::new(1.5, Palette::RED_SPORT))
                .rounding(12.0)
                .inner_margin(16.0),
        )
        .show(ctx, |ui| {
            ui.set_min_width(270.0);
            ui.label(
                egui::RichText::new(format!("Voulez-vous supprimer « {} » ?", task_title))
                    .strong()
                    .color(Palette::TEXT_PRIMARY),
            );
            ui.add_space(12.0);

            if is_recurring {
                // Option 1 : Supprimer uniquement l'occurrence de ce jour
                if ui
                    .add(
                        egui::Button::new("Uniquement aujourd'hui")
                            .min_size(egui::vec2(240.0, 28.0)),
                    )
                    .clicked()
                {
                    let date_str = app.selected_date.format("%Y-%m-%d").to_string();
                    if let Some(t) = app.save.custom_tasks.iter_mut().find(|t| t.id == task_id) {
                        if !t.skipped_dates.contains(&date_str) {
                            t.skipped_dates.push(date_str);
                        }
                    }
                    app.save.save();
                    app.task_to_delete = None;
                }

                ui.add_space(6.0);

                // Option 2 : Supprimer toute la série récurrente
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Toute la série récurrente")
                                .color(egui::Color32::WHITE),
                        )
                        .fill(Palette::RED_SPORT)
                        .min_size(egui::vec2(240.0, 28.0)),
                    )
                    .clicked()
                {
                    app.save.custom_tasks.retain(|t| t.id != task_id);
                    app.save
                        .completed_custom_occurrences
                        .retain(|k| !k.starts_with(&task_id));
                    app.save.save();
                    app.task_to_delete = None;
                }
            } else {
                // Activité unique : suppression directe
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Supprimer").color(egui::Color32::WHITE),
                            )
                            .fill(Palette::RED_SPORT)
                            .rounding(8.0),
                        )
                        .clicked()
                    {
                        app.save.custom_tasks.retain(|t| t.id != task_id);
                        app.save
                            .completed_custom_occurrences
                            .retain(|k| !k.starts_with(&task_id));
                        app.save.save();
                        app.task_to_delete = None;
                    }

                    if ui.button("Annuler").clicked() {
                        app.task_to_delete = None;
                    }
                });
            }
        });

    if !open {
        app.task_to_delete = None;
    }
}

/// Fenêtre flottante soignée permettant de paramétrer la tâche créée par clic droit
fn render_new_task_modal(app: &mut DuckApp, ctx: &egui::Context) {
    if !app.new_task.open {
        return;
    }

    let mut is_open = app.new_task.open;
    egui::Window::new("➕ Nouvelle activité")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Palette::BG_CARD)
                .stroke(egui::Stroke::new(1.5_f32, Palette::GOLD_ACCENT))
                .rounding(14.0)
                .inner_margin(18.0),
        )
        .show(ctx, |ui| {
            ui.set_min_width(330.0);

            // --- 1. NOM DE L'ACTIVITÉ ---
            ui.label(
                egui::RichText::new("Nom de l'activité")
                    .size(12.0)
                    .color(Palette::TEXT_MUTED),
            );
            ui.add_space(3.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.new_task.title)
                    .hint_text("Ex: Vélo d'appart, Banjo, Révision...")
                    .margin(egui::Margin::symmetric(10.0, 7.0))
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(12.0);

            // --- 2. TABLEAU ALIGNÉ : DÉBUT (1 SEUL BLOC) & DURÉE ---
            egui::Grid::new("new_task_time_grid")
                .num_columns(2)
                .spacing([14.0, 10.0])
                .show(ui, |ui| {
                    // Ligne 1 : Début dans un seul bloc unifié [ 18 : 30 ]
                    ui.label(
                        egui::RichText::new("Début")
                            .size(12.5)
                            .color(Palette::TEXT_MUTED),
                    );

                    egui::Frame::none()
                        .fill(Palette::BG_MAIN)
                        .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.add(
                                    egui::DragValue::new(&mut app.new_task.start_hour)
                                        .range(7..=21)
                                        .custom_formatter(|n, _| format!("{:02}", n as u32)),
                                );
                                ui.label(
                                    egui::RichText::new(":")
                                        .strong()
                                        .color(Palette::TEXT_PRIMARY),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut app.new_task.start_minute)
                                        .range(0..=59)
                                        .speed(5.0)
                                        .custom_formatter(|n, _| format!("{:02}", n as u32)),
                                );
                            });
                        });
                    ui.end_row();

                    // Ligne 2 : Durée (valeur manuelle mise en avant + pilules presets)
                    ui.label(
                        egui::RichText::new("Durée")
                            .size(12.5)
                            .color(Palette::TEXT_MUTED),
                    );

                    ui.horizontal(|ui| {
                        // Boîte principale pour la saisie manuelle
                        egui::Frame::none()
                            .fill(Palette::BG_MAIN)
                            .stroke(egui::Stroke::new(1.0_f32, Palette::GOLD_ACCENT))
                            .rounding(8.0)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::DragValue::new(&mut app.new_task.duration_minutes)
                                        .range(10..=480)
                                        .speed(5.0)
                                        .suffix(" min"),
                                );
                            });

                        ui.add_space(4.0);

                        // Petits boutons presets bien distincts visuellement
                        ui.spacing_mut().item_spacing.x = 4.0;
                        for preset in [30, 45, 60, 90] {
                            let is_active = app.new_task.duration_minutes == preset;
                            let fill = if is_active {
                                egui::Color32::from_rgba_unmultiplied(217, 155, 56, 45)
                            } else {
                                egui::Color32::from_rgb(44, 41, 37)
                            };
                            let txt_col = if is_active {
                                Palette::GOLD_ACCENT
                            } else {
                                Palette::TEXT_MUTED
                            };

                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!("{}m", preset))
                                            .size(11.0)
                                            .color(txt_col),
                                    )
                                    .fill(fill)
                                    .rounding(6.0),
                                )
                                .clicked()
                            {
                                app.new_task.duration_minutes = preset;
                            }
                        }
                    });
                    ui.end_row();
                });

            ui.add_space(12.0);

            // --- 3. CATÉGORIE / COULEUR (PILULES SANS CARRÉS NI RONDS ASCII) ---
            ui.label(
                egui::RichText::new("Catégorie")
                    .size(12.0)
                    .color(Palette::TEXT_MUTED),
            );
            ui.add_space(4.0);

            let colors = [
                TaskColor::RedSport,
                TaskColor::PurpleOther,
                TaskColor::GreenCare,
                TaskColor::GoldPerso,
                TaskColor::BlueStudy,
            ];

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                for col in colors {
                    let selected = app.new_task.color == col;
                    let accent = col.to_color32();
                    let bg = if selected {
                        egui::Color32::from_rgba_unmultiplied(
                            accent.r(),
                            accent.g(),
                            accent.b(),
                            55,
                        )
                    } else {
                        Palette::BG_MAIN
                    };
                    let stroke = if selected {
                        egui::Stroke::new(1.5_f32, accent)
                    } else {
                        egui::Stroke::new(1.0_f32, Palette::BORDER)
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(col.label())
                            .size(12.0)
                            .strong()
                            .color(if selected {
                                Palette::TEXT_PRIMARY
                            } else {
                                accent
                            }),
                    )
                    .fill(bg)
                    .stroke(stroke)
                    .rounding(8.0);

                    if ui.add(btn).clicked() {
                        app.new_task.color = col;
                    }
                }
            });

            ui.add_space(12.0);

            // --- 4. RÉPÉTITION (BOUTONS SEGMENTS PROPRES) ---
            ui.label(
                egui::RichText::new("Répétition")
                    .size(12.0)
                    .color(Palette::TEXT_MUTED),
            );
            ui.add_space(4.0);

            let modes = [
                (RepeatMode::Once, "1 fois"),
                (RepeatMode::EveryWeek, "Chaque sem."),
                (RepeatMode::EveryTwoWeeks, "1 sem. / 2"),
            ];

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for (mode, short_lbl) in modes {
                    let selected = app.new_task.repeat_mode == mode;
                    let fill = if selected {
                        egui::Color32::from_rgba_unmultiplied(217, 155, 56, 45)
                    } else {
                        Palette::BG_MAIN
                    };
                    let stroke = if selected {
                        egui::Stroke::new(1.2_f32, Palette::GOLD_ACCENT)
                    } else {
                        egui::Stroke::new(1.0_f32, Palette::BORDER)
                    };
                    let col = if selected {
                        Palette::GOLD_ACCENT
                    } else {
                        Palette::TEXT_MUTED
                    };

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(short_lbl).size(11.5).color(col))
                                .fill(fill)
                                .stroke(stroke)
                                .rounding(8.0),
                        )
                        .clicked()
                    {
                        app.new_task.repeat_mode = mode;
                    }
                }
            });

            if app.new_task.repeat_mode != RepeatMode::Once {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Nombre de répétitions :")
                            .size(12.0)
                            .color(Palette::TEXT_MUTED),
                    );
                    ui.add(
                        egui::DragValue::new(&mut app.new_task.repeat_count)
                            .range(1..=52)
                            .suffix(" fois"),
                    );
                });
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // --- 5. BOUTONS D'ACTION ---
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("  Ajouter  ")
                                .strong()
                                .color(Palette::BG_MAIN),
                        )
                        .fill(Palette::GREEN_ACCENT)
                        .rounding(8.0),
                    )
                    .clicked()
                {
                    let clean_title = if app.new_task.title.trim().is_empty() {
                        "Nouvelle activité".to_string()
                    } else {
                        app.new_task.title.trim().to_string()
                    };

                    let unique_id = format!(
                        "custom_{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis())
                            .unwrap_or(0)
                    );

                    let task = CustomTask {
                        id: unique_id,
                        title: clean_title,
                        start_date: app.selected_date.format("%Y-%m-%d").to_string(),
                        start_hour: app.new_task.start_hour,
                        start_minute: app.new_task.start_minute,
                        duration_minutes: app.new_task.duration_minutes,
                        color: app.new_task.color,
                        repeat_mode: app.new_task.repeat_mode,
                        repeat_count: if app.new_task.repeat_mode == RepeatMode::Once {
                            1
                        } else {
                            app.new_task.repeat_count.max(1)
                        },
                        skipped_dates: Vec::new(),
                    };

                    app.save.custom_tasks.push(task);
                    app.save.save();
                    app.new_task.open = false;
                }

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("  Annuler  ").color(Palette::TEXT_MUTED),
                        )
                        .fill(Palette::BG_MAIN)
                        .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
                        .rounding(8.0),
                    )
                    .clicked()
                {
                    app.new_task.open = false;
                }
            });
        });

    if !is_open {
        app.new_task.open = false;
    }
}

/// Dessine une carte colorée avec liseré gauche dans la chronologie
fn draw_timeline_block(
    painter: &egui::Painter,
    track_left: f32,
    track_right: f32,
    y1: f32,
    y2: f32,
    accent: egui::Color32,
    label: &str,
) {
    let rect = egui::Rect::from_min_max(
        egui::pos2(track_left + 4.0, y1),
        egui::pos2(track_right - 4.0, y2),
    );
    let bg_tint = egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 52);

    painter.rect_filled(rect, 6.0, bg_tint);
    painter.rect_filled(
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.left() + 4.0, rect.bottom())),
        2.0,
        accent,
    );
    painter.text(
        rect.min + egui::vec2(12.0, 5.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(11.5),
        Palette::TEXT_PRIMARY,
    );
}
