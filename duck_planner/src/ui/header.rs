//! Composants d'en-tête, sélecteur de calendrier mensuel, grille de statistiques (Bento)
//! et barre de navigation inférieure.

use crate::app::{DuckApp, MainTab};
use crate::ics::fetch_all_courses;
use crate::theme::{french_date_label, Palette};
use chrono::{Datelike, Days, NaiveDate};
use eframe::egui;
use std::f32::consts::TAU;

/// Barre tout en haut avec bouton Calendrier cliquable, synchronisation Untis et épingle
/// Barre tout en haut avec bouton Calendrier cliquable, Coach IA, synchronisation Untis et épingle
pub fn render_top_bar(app: &mut DuckApp, ctx: &egui::Context, ui: &mut egui::Ui, today: NaiveDate) {
    ui.horizontal(|ui| {
        let cal_label = if app.selected_date == today {
            "📅 Aujourd'hui".to_string()
        } else {
            format!("📅 {}", french_date_label(app.selected_date))
        };

        let btn_fill = if app.calendar_open {
            Palette::BG_CARD
        } else {
            egui::Color32::TRANSPARENT
        };

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(cal_label)
                        .color(Palette::TEXT_PRIMARY)
                        .strong(),
                )
                .fill(btn_fill)
                .rounding(8.0),
            )
            .on_hover_text("Ouvrir le calendrier pour choisir une date")
            .clicked()
        {
            app.calendar_open = !app.calendar_open;
            if app.calendar_open {
                app.calendar_month =
                    NaiveDate::from_ymd_opt(app.selected_date.year(), app.selected_date.month(), 1)
                        .unwrap_or(app.selected_date);
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 1. Bouton Épingle (Always on Top)
            let pin_col = if app.always_on_top {
                Palette::CORAL_NOW
            } else {
                Palette::TEXT_MUTED
            };
            if ui
                .add(egui::Button::new(egui::RichText::new("📌").color(pin_col)).frame(false))
                .on_hover_text("Garder au premier plan")
                .clicked()
            {
                app.always_on_top = !app.always_on_top;
                let level = if app.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                };
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
            }

            // 2. Bouton Synchronisation Untis
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("🔄").color(Palette::TEXT_MUTED))
                        .frame(false),
                )
                .on_hover_text("Synchroniser Untis")
                .clicked()
            {
                app.all_courses = fetch_all_courses(&app.config.ics_url);
            }

            ui.add_space(4.0);

            // 3. Bouton Coach IA (ouvre la fenêtre de génération/adaptation)
            let ai_bg = if app.ai_modal_open {
                egui::Color32::from_rgba_unmultiplied(160, 120, 220, 60)
            } else {
                Palette::BG_CARD
            };
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("✨ IA")
                            .size(12.0)
                            .strong()
                            .color(Palette::PURPLE_REWARD),
                    )
                    .fill(ai_bg)
                    .stroke(egui::Stroke::new(1.0_f32, Palette::PURPLE_REWARD))
                    .rounding(8.0),
                )
                .on_hover_text("Ouvrir l'assistant planning IA")
                .clicked()
            {
                app.ai_modal_open = !app.ai_modal_open;
            }
        });
    });
}
/// Calendrier mensuel dépliable permettant de cliquer sur n'importe quel jour
pub fn render_calendar_popup(app: &mut DuckApp, ui: &mut egui::Ui, today: NaiveDate) {
    let month_name = match app.calendar_month.month() {
        1 => "Janvier",
        2 => "Février",
        3 => "Mars",
        4 => "Avril",
        5 => "Mai",
        6 => "Juin",
        7 => "Juillet",
        8 => "Août",
        9 => "Septembre",
        10 => "Octobre",
        11 => "Novembre",
        _ => "Décembre",
    };

    egui::Frame::none()
        .fill(Palette::BG_CARD)
        .stroke(egui::Stroke::new(1.0_f32, Palette::GOLD_ACCENT))
        .rounding(12.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.small_button("‹").clicked() {
                    let (y, m) = if app.calendar_month.month() == 1 {
                        (app.calendar_month.year() - 1, 12)
                    } else {
                        (app.calendar_month.year(), app.calendar_month.month() - 1)
                    };
                    if let Some(d) = NaiveDate::from_ymd_opt(y, m, 1) {
                        app.calendar_month = d;
                    }
                }

                ui.label(
                    egui::RichText::new(format!("{} {}", month_name, app.calendar_month.year()))
                        .strong()
                        .color(Palette::TEXT_PRIMARY),
                );

                if ui.small_button("›").clicked() {
                    let (y, m) = if app.calendar_month.month() == 12 {
                        (app.calendar_month.year() + 1, 1)
                    } else {
                        (app.calendar_month.year(), app.calendar_month.month() + 1)
                    };
                    if let Some(d) = NaiveDate::from_ymd_opt(y, m, 1) {
                        app.calendar_month = d;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Aujourd'hui").clicked() {
                        app.selected_date = today;
                        app.calendar_month =
                            NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                                .unwrap_or(today);
                        app.calendar_open = false;
                    }
                });
            });

            ui.add_space(6.0);

            egui::Grid::new("calendar_days_grid")
                .num_columns(7)
                .spacing([6.0, 4.0])
                .show(ui, |ui| {
                    for day_hdr in ["Lu", "Ma", "Me", "Je", "Ve", "Sa", "Di"] {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(day_hdr)
                                    .size(11.0)
                                    .color(Palette::TEXT_MUTED),
                            );
                        });
                    }
                    ui.end_row();

                    let first_day = app.calendar_month;
                    let start_offset = first_day.weekday().num_days_from_monday();

                    for _ in 0..start_offset {
                        ui.label("");
                    }

                    let mut current = first_day;
                    let mut col = start_offset;

                    while current.month() == first_day.month() {
                        let is_selected = current == app.selected_date;
                        let is_curr_today = current == today;

                        let fill = if is_selected {
                            Palette::GOLD_ACCENT
                        } else if is_curr_today {
                            egui::Color32::from_rgb(55, 52, 46)
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let txt_col = if is_selected {
                            Palette::BG_MAIN
                        } else if is_curr_today {
                            Palette::GREEN_ACCENT
                        } else {
                            Palette::TEXT_PRIMARY
                        };

                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{:2}", current.day()))
                                .size(12.0)
                                .color(txt_col),
                        )
                        .fill(fill)
                        .rounding(6.0);

                        if ui.add(btn).clicked() {
                            app.selected_date = current;
                            app.calendar_open = false;
                        }

                        col += 1;
                        if col == 7 {
                            col = 0;
                            ui.end_row();
                        }

                        current = current.checked_add_days(Days::new(1)).unwrap_or(current);
                        if current == first_day {
                            break;
                        }
                    }
                });
        });
}

/// Titre du jour affiché et flèches `<` / `>` pour naviguer dans la semaine
pub fn render_date_selector(app: &mut DuckApp, ui: &mut egui::Ui, is_today: bool) {
    ui.horizontal(|ui| {
        let title = if is_today { "Aujourd'hui" } else { "Planning" };
        ui.label(
            egui::RichText::new(title)
                .size(22.0)
                .strong()
                .color(Palette::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(french_date_label(app.selected_date))
                .size(14.0)
                .color(Palette::TEXT_MUTED),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("›")
                            .size(20.0)
                            .color(Palette::TEXT_MUTED),
                    )
                    .frame(false),
                )
                .clicked()
            {
                app.selected_date = app
                    .selected_date
                    .checked_add_days(Days::new(1))
                    .unwrap_or(app.selected_date);
            }
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("‹")
                            .size(20.0)
                            .color(Palette::TEXT_MUTED),
                    )
                    .frame(false),
                )
                .clicked()
            {
                app.selected_date = app
                    .selected_date
                    .checked_sub_days(Days::new(1))
                    .unwrap_or(app.selected_date);
            }
        });
    });
}

/// Grille Bento en haut : Progression du jour (%) à gauche, 3 cartes compteurs à droite
pub fn render_bento_stats(
    ui: &mut egui::Ui,
    avail_w: f32,
    courses_count: usize,
    done_tasks: u32,
    total_tasks: u32,
) {
    let completion_pct = if total_tasks > 0 {
        ((done_tasks as f32 / total_tasks as f32) * 100.0).round() as u32
    } else {
        100
    };
    let remaining_tasks = total_tasks.saturating_sub(done_tasks);

    // Utilisation de `ui.columns(2, ...)` pour garantir que les 2 colonnes partagent exactement la largeur disponible
    ui.columns(2, |cols| {
        // --- COLONNE GAUCHE : Progression du jour (%) ---
        let col0_w = cols[0].available_width() - 26.0;
        egui::Frame::none()
            .fill(Palette::BG_CARD)
            .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
            .rounding(12.0)
            .inner_margin(12.0)
            .show(&mut cols[0], |ui| {
                ui.set_width(col0_w.max(120.0));
                ui.set_height(114.0);

                // Empilement vertical propre : Titre -> Graphique -> Pourcentage
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Progression du jour")
                            .size(12.0)
                            .color(Palette::TEXT_MUTED),
                    );

                    ui.add_space(4.0);

                    let (graph_rect, _) = ui.allocate_exact_size(
                        egui::vec2(col0_w.max(120.0), 48.0),
                        egui::Sense::hover(),
                    );
                    let p = ui.painter();
                    let progress_ratio = (completion_pct as f32 / 100.0).clamp(0.04, 1.0);

                    let pts = vec![
                        egui::pos2(graph_rect.left(), graph_rect.bottom() - 4.0),
                        egui::pos2(
                            graph_rect.left() + graph_rect.width() * 0.35,
                            graph_rect.bottom() - 18.0 * progress_ratio,
                        ),
                        egui::pos2(
                            graph_rect.left() + graph_rect.width() * 0.7,
                            graph_rect.bottom() - 34.0 * progress_ratio,
                        ),
                        egui::pos2(
                            graph_rect.right() - 4.0,
                            graph_rect.bottom() - 44.0 * progress_ratio,
                        ),
                    ];

                    let mut poly = pts.clone();
                    poly.push(egui::pos2(graph_rect.right() - 4.0, graph_rect.bottom()));
                    poly.push(egui::pos2(graph_rect.left(), graph_rect.bottom()));

                    p.add(egui::Shape::convex_polygon(
                        poly,
                        egui::Color32::from_rgba_unmultiplied(134, 184, 107, 35),
                        egui::Stroke::NONE,
                    ));
                    p.add(egui::Shape::line(
                        pts.clone(),
                        egui::Stroke::new(2.0_f32, Palette::GREEN_ACCENT),
                    ));
                    if let Some(last) = pts.last() {
                        p.circle_filled(*last, 3.5, Palette::GREEN_ACCENT);
                    }

                    ui.add_space(4.0);

                    ui.label(
                        egui::RichText::new(format!("{} %", completion_pct))
                            .size(20.0)
                            .strong()
                            .color(Palette::GREEN_ACCENT),
                    );
                });
            });

        // --- COLONNE DROITE : 3 cartes compteurs bien cadrées ---
        let col1_w = (cols[1].available_width() - 26.0).max(120.0);
        let _ = avail_w; // paramètre conservé pour compatibilité

        cols[1].vertical(|ui| {
            stat_row_card(
                ui,
                col1_w,
                "Cours",
                &format!("{} cours", courses_count),
                Palette::BLUE_COURSE,
                None,
            );
            ui.add_space(6.0);
            stat_row_card(
                ui,
                col1_w,
                "Objectifs",
                &format!("{} / {}", done_tasks, total_tasks),
                Palette::GREEN_ACCENT,
                Some(completion_pct as f32 / 100.0),
            );
            ui.add_space(6.0);
            stat_row_card(
                ui,
                col1_w,
                "Restant",
                &format!("{} act.", remaining_tasks),
                Palette::GOLD_ACCENT,
                None,
            );
        });
    });
}

/// Petite carte rectangulaire horizontale à largeur bornée pour éviter tout débordement
fn stat_row_card(
    ui: &mut egui::Ui,
    width: f32,
    label: &str,
    value: &str,
    value_color: egui::Color32,
    ring_progress: Option<f32>,
) {
    egui::Frame::none()
        .fill(Palette::BG_CARD)
        .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
        .rounding(10.0)
        .inner_margin(egui::Margin::symmetric(12.0, 9.5))
        .show(ui, |ui| {
            // allocate_ui_with_layout borne strictement la largeur à `width`
            ui.allocate_ui_with_layout(
                egui::vec2(width, 20.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .size(12.5)
                            .color(Palette::TEXT_MUTED),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(value)
                                .size(13.0)
                                .strong()
                                .color(value_color),
                        );
                        if let Some(prog) = ring_progress {
                            draw_ring_gauge(ui, prog, value_color);
                        }
                    });
                },
            );
        });
}

/// Dessine la jauge circulaire de pourcentage
fn draw_ring_gauge(ui: &mut egui::Ui, progress: f32, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let radius = 7.0;

    painter.circle_stroke(
        center,
        radius,
        egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(55, 52, 48)),
    );

    let steps = (progress.clamp(0.0, 1.0) * 28.0) as usize;
    if steps > 1 {
        let mut pts = Vec::with_capacity(steps);
        for i in 0..=steps {
            let angle = -TAU / 4.0 + (i as f32 / 28.0) * TAU;
            pts.push(center + egui::vec2(angle.cos() * radius, angle.sin() * radius));
        }
        painter.add(egui::Shape::line(pts, egui::Stroke::new(2.5_f32, color)));
    }
}

/// Barre de navigation principale en bas de l'écran (2 onglets parfaitement centrés)
pub fn render_bottom_nav(app: &mut DuckApp, ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(Palette::BG_CARD)
        .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
        .rounding(14.0)
        .inner_margin(egui::Margin::symmetric(8.0, 8.0))
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                nav_item(
                    &mut cols[0],
                    "🍅",
                    "Chronologie",
                    MainTab::Chronologie,
                    &mut app.active_tab,
                    Palette::GREEN_ACCENT,
                );
                nav_item(
                    &mut cols[1],
                    "📋",
                    "Tâches",
                    MainTab::Taches,
                    &mut app.active_tab,
                    Palette::GOLD_ACCENT,
                );
            });
        });
}

/// Bouton d'onglet centré (icône + libellé + point indicateur)
fn nav_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    tab: MainTab,
    current: &mut MainTab,
    dot_col: egui::Color32,
) {
    let col_w = ui.available_width();
    let active = *current == tab;
    let txt_col = if active {
        Palette::TEXT_PRIMARY
    } else {
        Palette::TEXT_MUTED
    };

    // Zone cliquable occupant toute la largeur de la colonne pour centrer parfaitement
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(col_w, 42.0), egui::Sense::click());
    if resp.clicked() {
        *current = tab;
    }

    let p = ui.painter();
    let cx = rect.center().x;

    // Icône centrée en haut
    p.text(
        egui::pos2(cx, rect.top() + 10.0),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        txt_col,
    );

    // Libellé centré au milieu
    p.text(
        egui::pos2(cx, rect.top() + 26.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        txt_col,
    );

    // Point lumineux centré en bas si l'onglet est actif
    if active {
        p.circle_filled(egui::pos2(cx, rect.bottom() - 3.0), 2.5, dot_col);
    }
}
