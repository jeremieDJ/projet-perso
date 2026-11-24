//! Assistant IA de planification (propulsé par Groq / openai/gpt-oss-120b)
//! avec gestion des priorités et des transports publics.

use crate::ics::CourseEvent;
use crate::models::{AppConfig, CustomTask, RepeatMode, TaskColor, TaskPriority};
use crate::transport::build_week_availability_summary;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTaskItem {
    pub title: String,
    pub date: String, // "YYYY-MM-DD"
    pub start_hour: u32,
    pub start_minute: u32,
    pub duration_minutes: u32,
    pub category: String,   // "sport", "soin", "medical", "etudes", "autre"
    pub priority_level: u8, // 1 = Flexible, 2 = Moyenne, 3 = Haute, 4 = Fixe
    pub repeat_weeks: u32,  // 1 = ponctuel, 4 = 4 semaines
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCancelItem {
    pub task_id: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPlanActions {
    pub assistant_message: String,
    #[serde(default)]
    pub cancel_occurrences: Vec<AiCancelItem>,
    #[serde(default)]
    pub new_tasks: Vec<AiTaskItem>,
}

/// Appelle l'API Groq en lui fournissant les cours (P4 fixes), les trajets CFF/CarPostal
/// et les tâches existantes avec leur niveau de priorité (P1 à P4).
pub fn ask_ai_planner(
    config: &AppConfig,
    user_prompt: &str,
    today: NaiveDate,
    all_courses: &[CourseEvent],
    current_tasks: &[CustomTask],
) -> Result<AiPlanActions, String> {
    if config.gemini_api_key.trim().is_empty() || config.gemini_api_key.contains("METS_TA_CLE") {
        return Err("Colle ta clé Groq (gsk_...) dans 'gemini_api_key' de config.json.".into());
    }

    // 1. Disponibilités (matin et soir) après cours et transport en commun sur 7 jours
    let availability = build_week_availability_summary(all_courses, today, config);

    // 2. Liste détaillée des tâches existantes sur les 7 prochains jours avec leur ID et priorité
    let mut existing_str = String::new();
    for i in 0..7 {
        let d = today + chrono::Duration::days(i);
        for t in current_tasks {
            if t.occurs_on(d) {
                existing_str.push_str(&format!(
                    "- Date: {} | ID: \"{}\" | Titre: \"{}\" | Horaire: {:02}:{:02} ({} min) | Priorité: {:?} (niveau {})\n",
                    d.format("%Y-%m-%d"),
                    t.id,
                    t.title,
                    t.start_hour,
                    t.start_minute,
                    t.duration_minutes,
                    t.priority,
                    t.priority as u8
                ));
            }
        }
    }

    let system_prompt = format!(
        "Tu es un assistant d'agenda intelligent.\n\
        Date d'aujourd'hui : {} ({}).\n\n\
        RÈGLES DE PRIORITÉ STRICTES (À RESPECTER IMPÉRATIVEMENT) :\n\
        - Niveau 4 (Fixe) : Les COURS universitaires sont intouchables. Tu ne peux JAMAIS les déplacer ni les supprimer.\n\
        - Niveau 3 (Haute) : Rendez-vous médical, examen, impératif. Si l'utilisateur ajoute un RDV médical ou un impératif qui chevauche une tâche de niveau 1 ou 2, tu DOIS annuler l'occurrence de cette tâche via `cancel_occurrences` (avec son `task_id` et la `date`) et la replanifier plus tard dans la soirée (après le RDV, avant 21h30) ou un autre jour libre dans `new_tasks`.\n\
        - Niveau 2 (Moyenne) : Soins des cheveux / douche. Déplacé uniquement si conflit avec niveau 3 ou 4.\n\
        - Niveau 1 (Flexible) : Sport (vélo d'appartement, muscu haut du corps), loisirs. Déplacé/annulé en premier dès qu'il y a un RDV médical (niveau 3) ou une soirée/fête.\n\n\
        RÈGLES SPORT, CHEVEUX & TRANSPORTS :\n\
        1. Sport (perte de poids) : Vélo d'appartement (cardio/fractionné doux) + renforcement haut du corps / gainage. INTERDICTION TOTALE de course à pied, sauts ou exercices lourds sur les genoux.\n\
        2. Soin cheveux : Respecte scrupuleusement la routine demandée (lavages complets, refresh matin, et co-wash le mercredi matin si MATIN LIBRE sinon le mercredi soir).\n\
        3. Transports : Ne place JAMAIS d'activité à la maison avant l'heure d'arrivée en transport le soir, ni après l'heure de départ indiquée le matin.\n\n\
        RÈGLE DE COMPLÉTUDE OBLIGATOIRE :\n\
        Si l'utilisateur demande un planning hebdomadaire ou sur 7 jours, tu DOIS impérativement générer les tâches pour CHACUN des 7 jours listés ci-dessous (du jour 1 au jour 7, soit environ 10 à 12 tâches au total dans `new_tasks`). Ne t'arrête JAMAIS après 2 jours.\n\n\
        COURS (NIVEAU 4 FIXE) & CRÉNEAUX LIBRES MAISON SUR 7 JOURS :\n{}\n\
        TÂCHES PERSONNALISÉES DÉJÀ PLANIFIÉES SUR 7 JOURS :\n{}\n\
        DEMANDE DE L'UTILISATEUR : \"{}\"\n\n\
        Réponds UNIQUEMENT en JSON valide selon ce schéma exact :\n\
        {{\n\
          \"assistant_message\": \"Explication courte des ajustements faits\",\n\
          \"cancel_occurrences\": [\n\
            {{ \"task_id\": \"custom_123\", \"date\": \"YYYY-MM-DD\" }}\n\
          ],\n\
          \"new_tasks\": [\n\
            {{\n\
              \"title\": \"Cheveux : Lavage complet (Syoss + Clere + Pantene)\",\n\
              \"date\": \"YYYY-MM-DD\",\n\
              \"start_hour\": 18,\n\
              \"start_minute\": 30,\n\
              \"duration_minutes\": 30,\n\
              \"category\": \"soin\",\n\
              \"priority_level\": 2,\n\
              \"repeat_weeks\": 1\n\
            }}\n\
          ]\n\
        }}",
        today.format("%Y-%m-%d"),
        today.weekday(),
        availability,
        if existing_str.is_empty() {
            "Aucune tâche personnalisée planifiée.".to_string()
        } else {
            existing_str
        },
        user_prompt
    );

    let url = "https://api.groq.com/openai/v1/chat/completions";

    let body = serde_json::json!({
        "model": "openai/gpt-oss-120b",
        "temperature": 0.2,
        "max_tokens": 4096,
        "messages": [
            {
                "role": "system",
                "content": "Tu réponds uniquement avec un objet JSON valide, sans texte ni balises markdown autour. Tu dois générer l'intégralité des tâches demandées sur tous les jours demandés."
            },
            {
                "role": "user",
                "content": system_prompt
            }
        ]
    });

    let resp = match ureq::post(url)
        .set(
            "Authorization",
            &format!("Bearer {}", config.gemini_api_key.trim()),
        )
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
    {
        Ok(r) => r,
        Err(ureq::Error::Status(code, response)) => {
            let err_body = response.into_string().unwrap_or_default();
            return Err(format!("Erreur Groq HTTP {} : {}", code, err_body));
        }
        Err(e) => return Err(format!("Erreur réseau API Groq : {}", e)),
    };

    let raw_resp = resp
        .into_string()
        .map_err(|e| format!("Erreur lecture réponse : {}", e))?;

    let json_val: serde_json::Value =
        serde_json::from_str(&raw_resp).map_err(|e| format!("Erreur parsing JSON : {}", e))?;

    let raw_text = json_val["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Réponse vide de l'IA")?;

    // Nettoie les éventuelles balises ```json ... ```
    let clean_json = raw_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str::<AiPlanActions>(clean_json)
        .map_err(|e| format!("JSON invalide : {} (brut: {})", e, clean_json))
}

/// Applique les annulations/décalages et ajoute les nouvelles tâches dans `save.custom_tasks`
pub fn apply_ai_actions(actions: &AiPlanActions, tasks: &mut Vec<CustomTask>) {
    for cancel in &actions.cancel_occurrences {
        if let Some(t) = tasks.iter_mut().find(|t| t.id == cancel.task_id) {
            if t.priority != TaskPriority::Fixed && !t.skipped_dates.contains(&cancel.date) {
                t.skipped_dates.push(cancel.date.clone());
            }
        }
    }

    let base_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    for (idx, item) in actions.new_tasks.iter().enumerate() {
        let color = match item.category.to_lowercase().as_str() {
            "sport" => TaskColor::RedSport,
            "soin" => TaskColor::GreenCare,
            "medical" | "médical" => TaskColor::GoldPerso,
            "etudes" | "études" => TaskColor::BlueStudy,
            _ => TaskColor::PurpleOther,
        };

        let priority = match item.priority_level {
            4 => TaskPriority::Fixed,
            3 => TaskPriority::High,
            2 => TaskPriority::Medium,
            _ => TaskPriority::Flexible,
        };

        tasks.push(CustomTask {
            id: format!("ai_{}_{}", base_ts, idx),
            title: item.title.clone(),
            start_date: item.date.clone(),
            start_hour: item.start_hour.clamp(7, 21),
            start_minute: item.start_minute.min(59),
            duration_minutes: item.duration_minutes.clamp(15, 240),
            color,
            priority,
            repeat_mode: if item.repeat_weeks > 1 {
                RepeatMode::EveryWeek
            } else {
                RepeatMode::Once
            },
            repeat_count: item.repeat_weeks.max(1),
            skipped_dates: Vec::new(),
        });
    }
}
