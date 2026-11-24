//! Interrogation de l'API officielle suisse `transport.opendata.ch`
//! pour calculer les créneaux libres (avant le premier cours et après le dernier cours).

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
    let ready_at = last_course_end + Duration::minutes(walk_minutes as i64);
    let date_str = ready_at.format("%Y-%m-%d").to_string();
    let time_str = ready_at.format("%H:%M").to_string();

    let req = ureq::get("[http://transport.opendata.ch/v1/connections](http://transport.opendata.ch/v1/connections)")
        .query("from", from_station)
        .query("to", to_station)
        .query("date", &date_str)
        .query("time", &time_str)
        .query("limit", "1");

    if let Ok(resp) = req.call() {
        if let Ok(data) = resp.into_json::<ConnectionsResponse>() {
            if let Some(conn) = data.connections.first() {
                if let Some(arr_str) = &conn.to.arrival {
                    if arr_str.len() >= 19 {
                        if let Ok(arr_dt) =
                            NaiveDateTime::parse_from_str(&arr_str[..19], "%Y-%m-%dT%H:%M:%S")
                        {
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

/// Formate un résumé complet (heure de départ le matin + heure de retour le soir) sur 7 jours
pub fn build_week_availability_summary(
    all_courses: &[crate::ics::CourseEvent],
    start_date: NaiveDate,
    config: &crate::models::AppConfig,
) -> String {
    let mut summary = String::new();

    for i in 0..7 {
        let date = start_date + Duration::days(i);
        let mut day_courses = crate::ics::courses_for_date(all_courses, date);
        day_courses.sort_by_key(|c| c.start);

        if let (Some(first), Some(last)) = (day_courses.first(), day_courses.last()) {
            // Marge de 1h15 avant le 1er cours pour le trajet Mollens -> Sion
            let leave_home = first.start - Duration::minutes(75);
            let home_time = estimate_home_arrival(
                &config.school_station,
                &config.home_station,
                last.end,
                config.walk_to_station_minutes,
            );

            let morning_status = if first.start.hour() >= 12 {
                format!(
                    "MATIN LIBRE (départ maison à {:02}:{:02})",
                    leave_home.hour(),
                    leave_home.minute()
                )
            } else {
                format!(
                    "Départ maison tôt à {:02}:{:02}",
                    leave_home.hour(),
                    leave_home.minute()
                )
            };

            summary.push_str(&format!(
                "- {} ({}) : cours de {:02}:{:02} à {:02}:{:02} | {} | Arrivée maison soir à {:02}:{:02} (libre après)\n",
                date.format("%Y-%m-%d"),
                date.weekday(),
                first.start.hour(),
                first.start.minute(),
                last.end.hour(),
                last.end.minute(),
                morning_status,
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
