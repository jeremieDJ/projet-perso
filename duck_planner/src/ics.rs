//! Téléchargement et analyse (parsing) du calendrier iCal (.ics) PocketCampus / Untis.
//! Applique également les filtres personnalisés (suppression et renommage de cours).

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Weekday};
use std::fs;

/// Représente un bloc de cours extrait du calendrier
#[derive(Debug, Clone)]
pub struct CourseEvent {
    pub uid: String,
    pub summary: String,
    pub location: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

/// Télécharge le flux `.ics` en ligne (avec sauvegarde locale de secours `schedule_cache.ics`)
pub fn fetch_all_courses(url: &str) -> Vec<CourseEvent> {
    let raw_ics = match ureq::get(url).call() {
        Ok(resp) => {
            if let Ok(text) = resp.into_string() {
                let _ = fs::write("schedule_cache.ics", &text);
                text
            } else {
                fs::read_to_string("schedule_cache.ics").unwrap_or_default()
            }
        }
        // En cas de coupure réseau, on lit le dernier cache local
        Err(_) => fs::read_to_string("schedule_cache.ics").unwrap_or_default(),
    };

    parse_ics(&raw_ics)
}

/// Filtre et trie chronologiquement les cours correspondant à une date précise
pub fn courses_for_date(all: &[CourseEvent], target: NaiveDate) -> Vec<CourseEvent> {
    let mut list: Vec<CourseEvent> = all
        .iter()
        .filter(|e| e.start.date() == target)
        .cloned()
        .collect();
    list.sort_by_key(|e| e.start);
    list
}

/// Découpe le texte brut du fichier `.ics` et applique les règles métier personnalisées
fn parse_ics(content: &str) -> Vec<CourseEvent> {
    let mut events = Vec::new();
    // La norme RFC 5545 coupe les longues lignes avec un saut de ligne suivi d'un espace
    let unfolded = content.replace("\r\n ", "").replace("\n ", "");

    let mut in_event = false;
    let mut uid = String::new();
    let mut summary = String::new();
    let mut location = String::new();
    let mut dtstart: Option<NaiveDateTime> = None;
    let mut dtend: Option<NaiveDateTime> = None;

    for line in unfolded.lines() {
        let line = line.trim();
        if line == "BEGIN:VEVENT" {
            in_event = true;
            uid.clear();
            summary.clear();
            location.clear();
            dtstart = None;
            dtend = None;
        } else if line == "END:VEVENT" && in_event {
            in_event = false;
            if let (Some(start), Some(end)) = (dtstart, dtend) {
                let mut clean_summary = summary.replace("\\,", ",").replace("\\n", " ");
                let mut clean_location = location.replace("\\,", ",");

                // RÈGLE 1 : Ignorer Französisch 3 et Physique 2 du lundi fin d'après-midi
                if should_ignore_course(&clean_summary, start) {
                    continue;
                }

                // RÈGLE 2 : Renommer Communication en Anglais + salle dédiée
                if clean_summary.contains("Communication Langue")
                    || clean_summary.contains("Communication")
                {
                    clean_summary = "Anglais".to_string();
                    clean_location = "VS-ENP.21.N405 (Classe)".to_string();
                }

                // Génère un identifiant unique si le flux n'en fournit pas
                if uid.is_empty() {
                    uid = format!("{}_{}", clean_summary, start.format("%Y%m%d%H%M"));
                }

                events.push(CourseEvent {
                    uid: uid.clone(),
                    summary: clean_summary,
                    location: clean_location,
                    start,
                    end,
                });
            }
        } else if in_event {
            if let Some(val) = line.strip_prefix("UID:") {
                uid = val.to_string();
            } else if let Some(val) = line.strip_prefix("SUMMARY:") {
                summary = val.to_string();
            } else if let Some(val) = line.strip_prefix("LOCATION:") {
                location = val.to_string();
            } else if line.starts_with("DTSTART") {
                if let Some((_, val)) = line.split_once(':') {
                    dtstart = parse_ics_datetime(val);
                }
            } else if line.starts_with("DTEND") {
                if let Some((_, val)) = line.split_once(':') {
                    dtend = parse_ics_datetime(val);
                }
            }
        }
    }

    events
}

/// Vérifie si un cours doit être masqué du planning
fn should_ignore_course(summary: &str, start: NaiveDateTime) -> bool {
    if start.weekday() != Weekday::Mon {
        return false;
    }
    let lower = summary.to_lowercase();
    let is_afternoon_slot = (start.hour() == 16 && start.minute() >= 15) || (start.hour() == 17);

    is_afternoon_slot
        && (lower.contains("französisch 3")
            || lower.contains("franzosisch 3")
            || lower.contains("physique 2, complément")
            || lower.contains("physique"))
}

/// Convertit une date iCal (`20261123T081500Z` ou locale) en heure locale système
fn parse_ics_datetime(val: &str) -> Option<NaiveDateTime> {
    let clean = val.trim();
    if clean.ends_with('Z') {
        let utc =
            NaiveDateTime::parse_from_str(clean.trim_end_matches('Z'), "%Y%m%dT%H%M%S").ok()?;
        let local_dt = Local.from_utc_datetime(&utc);
        Some(local_dt.naive_local())
    } else {
        NaiveDateTime::parse_from_str(clean, "%Y%m%dT%H%M%S").ok()
    }
}
