//! Fenêtre de dialogue avec l'assistant IA pour générer et adapter le planning.

use crate::ai::{apply_ai_actions, ask_ai_planner};
use crate::app::DuckApp;
use crate::theme::Palette;
use chrono::Local;
use eframe::egui;
use std::sync::mpsc;
use std::thread;

pub fn render_ai_modal(app: &mut DuckApp, ctx: &egui::Context) {
    // Vérifie si le thread IA en arrière-plan a terminé sa réponse
    if let Some(rx) = &app.ai_receiver {
        if let Ok(result) = rx.try_recv() {
            app.ai_loading = false;
            app.ai_receiver = None;
            match result {
                Ok(actions) => {
                    app.ai_last_reply = actions.assistant_message.clone();
                    apply_ai_actions(&actions, &mut app.save.custom_tasks);
                    app.save.save();
                }
                Err(err) => {
                    app.ai_last_reply = format!("⚠️ {}", err);
                }
            }
        }
    }

    if !app.ai_modal_open {
        return;
    }

    let mut open = app.ai_modal_open;
    egui::Window::new("✨ Coach & Planning IA")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Palette::BG_CARD)
                .stroke(egui::Stroke::new(1.5_f32, Palette::PURPLE_REWARD))
                .rounding(14.0)
                .inner_margin(18.0),
        )
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.set_max_width(420.0);

            ui.label(
                egui::RichText::new("Que veux-tu planifier ou adapter ?")
                    .size(13.0)
                    .strong()
                    .color(Palette::TEXT_PRIMARY),
            );
            ui.add_space(6.0);

            // Raccourcis en 1 clic pour tes cas fréquents
            ui.horizontal_wrapped(|ui| {
                if ui.small_button("🚴 Programme Sport + Cheveux").clicked() {
                    app.ai_prompt = "Annule toutes mes tâches IA existantes via cancel_occurrences, puis génère mon planning complet pour CHACUN des 7 jours (repeat_weeks: 1) en respectant strictement mes cours et trajets :\n\n\
1. ROUTINE CHEVEUX LONGS (catégorie \"soin\", priorité 2) :\n\
- Dimanche soir (18h30) : Cheveux : Lavage complet (Syoss racines + Clere longueurs + Pantene) (30 min)\n\
- Lundi, Mardi, Jeudi, Samedi matin (avant départ) : Cheveux : Refresh (Eau + noisette de Pantene) (15 min)\n\
- Mercredi : Si j'ai MATIN LIBRE (cours l'après-midi ou pas de cours), place Cheveux : Co-wash (Clere seul + Pantene) (30 min) le MATIN avant départ. Sinon, place-le le SOIR tôt après retour.\n\
- Vendredi soir (tôt, après retour) : Cheveux : Lavage complet (Syoss racines + Clere longueurs + Pantene) (30 min)\n\n\
2. SPORT PERTE DE POIDS (catégorie \"sport\", priorité 1 — 25 ans, 1m85, 120 kg, ZÉRO impact genoux) :\n\
- Lundi soir : Sport : Vélo d'appart Endurance (5 min échauffement + 30 min modéré + 5 min calme) (40 min)\n\
- Mardi soir : Sport : Mobilité & Étirements (15 min dos, nuque et poignets) (15 min)\n\
- Mercredi soir : Sport : Vélo d'appart Fractionné assis (5 min échauffement + 8x [30s rapide / 1m15 lent] + 5 min calme) (30 min)\n\
- Jeudi soir : Sport : Haut du corps & Gainage (3 tours : 10 pompes inclinées + 10 dips chaise + 30s planche) (25 min)\n\
- Samedi matin : Sport : Vélo d'appart Cardio long (45 min endurance douce + étirements bras/dos) (45 min)".to_string();
                }
                if ui.small_button("🎉 Ce soir je fais la fête").clicked() {
                    app.ai_prompt = "Ce soir je fais la fête, je ne peux rien faire. Annule ou décale mes activités de ce soir.".to_string();
                }
                if ui.small_button("🩺 RDV Médical demain 18h").clicked() {
                    app.ai_prompt = "Ajoute un rendez-vous médical (priorité haute) demain à 18h00 pendant 1h et décale le sport s'il y en avait un.".to_string();
                }
                if ui.small_button("🗑️ Vider tâches IA").clicked() {
                    app.save.custom_tasks.retain(|t| !t.id.starts_with("ai_"));
                    app.save.save();
                    app.ai_last_reply = "Toutes les tâches générées par l'IA ont été supprimées.".to_string();
                }
            });

            ui.add_space(8.0);

            // Zone de texte multi-lignes déroulante
            egui::ScrollArea::vertical()
                .id_salt("ai_prompt_scroll")
                .max_height(140.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.ai_prompt)
                            .hint_text("Ex: Rajoute un RDV médical jeudi à 17h30 et décale le sport...")
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let btn_enabled = !app.ai_loading && !app.ai_prompt.trim().is_empty();
                if ui
                    .add_enabled(
                        btn_enabled,
                        egui::Button::new(
                            egui::RichText::new(if app.ai_loading {
                                "⏳ Calcul en cours..."
                            } else {
                                "✨ Générer / Adapter"
                            })
                            .strong()
                            .color(Palette::BG_MAIN),
                        )
                        .fill(Palette::PURPLE_REWARD)
                        .rounding(8.0),
                    )
                    .clicked()
                {
                    app.ai_loading = true;
                    app.ai_last_reply.clear();

                    let cfg = app.config.clone();
                    let prompt = app.ai_prompt.clone();
                    let today = Local::now().date_naive();
                    let courses = app.all_courses.clone();
                    let tasks = app.save.custom_tasks.clone();

                    let (tx, rx) = mpsc::channel();
                    app.ai_receiver = Some(rx);

                    thread::spawn(move || {
                        let res = ask_ai_planner(&cfg, &prompt, today, &courses, &tasks);
                        let _ = tx.send(res);
                    });
                }

                if ui.button("Fermer").clicked() {
                    app.ai_modal_open = false;
                }
            });

            if !app.ai_last_reply.is_empty() {
                ui.add_space(10.0);
                egui::Frame::none()
                    .fill(Palette::BG_MAIN)
                    .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
                    .rounding(8.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("ai_reply_scroll")
                            .max_height(100.0)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&app.ai_last_reply)
                                        .size(12.0)
                                        .color(Palette::TEXT_PRIMARY),
                                );
                            });
                    });
            }
        });

    if !open {
        app.ai_modal_open = false;
    }
}
