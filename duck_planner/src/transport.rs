//! Interrogation de l'API officielle suisse `transport.opendata.ch`
//! pour calculer l'heure exacte d'arrivée à la maison après le dernier cours.

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use serde::Deserialize;

#[derive(Deserialize)]
struct ConnectionsResponse {
    connections: Vec<Connection>,
}

#[derive(Deserialize)]
struct Connection {
    to: StopPoint,
}

#[derive(Deserialize)]
struct StopPoint {
    arrival: Option<String>, // Format ISO ex: "2026-11-24T17:12:00+0100"
}

/// Calcule l'heure d'arrivée à la maison après la fin du dernier cours
pub fn estimate_home_arrival(
    from_station: &str,
    to_station: &str,
    last_course_end: NaiveDateTime,
    walk_minutes: u32,
) -> NaiveDateTime {
    // Heure à laquelle tu es prêt à prendre le bus/train à l'arrêt
    let ready_at = last_course_end + Duration::minutes(walk_minutes as i64);
    let date_str = ready_at.format("%Y-%m-%d").to_string();
    let time_str = ready_at.format("%H:%M").to_string();

    let req = ureq::get("http://transport.opendata.ch/v1/connections")
        .query("from", from_station)
        .query("to", to_station)
        .query("date", &date_str)
        .query("time", &time_str)
        .query("limit", "1");

    if let Ok(resp) = req.call() {
        if let Ok(data) = resp.into_json::<ConnectionsResponse>() {
            if let Some(conn) = data.connections.first() {
                if let Some(arr_str) = &conn.to.arrival {
                    // Extrait "YYYY-MM-DDTHH:MM:SS" avant le décalage horaire "+0100"
                    if arr_str.len() >= 19 {
                        if let Ok(arr_dt) =
                            NaiveDateTime::parse_from_str(&arr_str[..19], "%Y-%m-%dT%H:%M:%S")
                        {
                            // + 10 min de marche de l'arrêt jusqu'à la porte de chez toi
                            return arr_dt + Duration::minutes(10);
                        }
                    }
                }
            }
        }
    }

    // Repli sécurisé si hors-ligne : fin des cours + 1h15 de trajet
    last_course_end + Duration::minutes(75)
}

/// Formate un résumé lisible des heures de retour à la maison sur 7 jours pour l'IA
pub fn build_week_availability_summary(
    all_courses: &[crate::ics::CourseEvent],
    start_date: NaiveDate,
    config: &crate::models::AppConfig,
) -> String {
    let mut summary = String::new();

    for i in 0..7 {
        let date = start_date + Duration::days(i);
        let day_courses = crate::ics::courses_for_date(all_courses, date);

        if let Some(last) = day_courses.last() {
            let home_time = estimate_home_arrival(
                &config.school_station,
                &config.home_station,
                last.end,
                config.walk_to_station_minutes,
            );
            summary.push_str(&format!(
                "- {} ({}) : cours jusqu'à {:02}:{:02} -> Arrivée maison en transport à {:02}:{:02} (libre après cette heure)\n",
                date.format("%Y-%m-%d"),
                date.weekday(),
                last.end.hour(),
                last.end.minute(),
                home_time.hour(),
                home_time.minute()
            ));
        } else {
            summary.push_str(&format!(
                "- {} ({}) : Aucun cours -> Libre toute la journée (dès 08:30)\n",
                date.format("%Y-%m-%d"),
                date.weekday()
            ));
        }
    }

    summary
}
