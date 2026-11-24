//! Point d'entrée principal de l'application Duck Planner.
//! Configure et lance la fenêtre native Windows via `eframe`.

// Masque la console noire Windows en mode release (`cargo run --release`)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Déclaration de tous les modules du projet
mod ai;
mod app;
mod ics;
mod models;
mod theme;
mod transport;
mod ui;

use app::DuckApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Configuration de la fenêtre native
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 830.0]) // Taille par défaut confortable
            .with_min_inner_size([480.0, 680.0]) // Taille minimale pour éviter tout chevauchement
            .with_always_on_top() // Épinglée au premier plan au démarrage
            .with_resizable(true), // Redimensionnable librement
        ..Default::default()
    };

    // Lancement de la boucle graphique
    eframe::run_native(
        "Duck Planner",
        options,
        Box::new(|_cc| Ok(Box::new(DuckApp::new()))),
    )
}
