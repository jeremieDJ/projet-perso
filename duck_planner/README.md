# 🦆 Duck Planner

Application bureau personnelle développée en **Rust** (`eframe` / `egui`) pour centraliser l'emploi du temps universitaire (synchronisation `.ics` PocketCampus / Untis), les routines quotidiennes (sport, soin capillaire) et un système de gamification (XP, niveaux et graines à échanger contre des récompenses).

---

## 📁 Structure du projet

```text
duck_planner/
├── Cargo.toml           # Dépendances et métadonnées du projet Rust
├── config.json          # Configuration utilisateur (URL .ics, routines sport & cheveux)
├── save.json            # Sauvegarde automatique de la progression (généré à l'exécution)
├── schedule_cache.ics   # Cache local du calendrier en cas de coupure réseau (généré)
├── README.md            # Documentation du projet
└── src/
    ├── main.rs          # Point d'entrée et configuration de la fenêtre native
    ├── app.rs           # État global de l'application et boucle de mise à jour
    ├── models.rs        # Structures de données et logique de sauvegarde / XP
    ├── ics.rs           # Téléchargement, lecture et filtrage du calendrier .ics
    ├── theme.rs         # Palette de couleurs et formatage des textes
    └── ui/              # Composants de l'interface graphique
        ├── mod.rs       # Exportation des sous-modules d'interface
        ├── header.rs    # Barre supérieure, navigation de date, grille Bento et barre du bas
        ├── timeline.rs  # Vue "Chronologie" (grille horaire verticale + ligne temps réel)
        └── tabs.rs      # Vues "Tâches" (cases à cocher + chrono) et "Récompenses"

🧩 Rôle détaillé de chaque fichier

  1. Fichiers de configuration et de données (à la racine)

    Cargo.toml
        Rôle : Manifeste du projet Rust géré par Cargo.
        
        Détails :
        
        Définit le nom du binaire (duck_planner), sa version et l'édition Rust.
        
        Déclare les bibliothèques externes : eframe (fenêtrage et rendu graphique egui), chrono (gestion des dates, heures et fuseaux horaires), serde / serde_json (lecture et écriture des fichiers .json) et ureq (requêtes HTTP pour télécharger l'emploi du temps).
        
    config.json
        Rôle : Fichier de paramétrage externe modifiable sans recompiler le code.
        
        Détails :
        
        ics_url : Adresse web du calendrier iCal Untis / PocketCampus.
        
        hair_routine : Dictionnaire associant chaque jour anglais (Monday à Sunday) à un titre et une liste de produits capillaires.
        
        sport_routine : Dictionnaire associant chaque jour à une séance (titre, description des exercices et durée en secondes timer_seconds pour le chronomètre intégré).
        
    save.json (généré automatiquement)
        Rôle : Persistance locale de la progression utilisateur.
        
        Détails :
        
        Enregistre le niveau actuel (level), les points d'expérience (xp), la monnaie virtuelle (seeds), la date du dernier lancement (last_date) ainsi que l'état des cases cochées (hair_done_today, sport_done_today, completed_courses).
        
        Réécrit sur le disque à chaque modification (cochage, décochage, achat de récompense ou passage à un nouveau jour).
        
    schedule_cache.ics (généré automatiquement)
        Rôle : Sauvegarde hors-ligne du planning universitaire.
        
        Détails :
        
        Copie brute du dernier fichier .ics téléchargé avec succès, utilisée automatiquement par ics.rs si l'ordinateur n'a pas accès à Internet au démarrage.
        
  2. Cœur logique de l'application (src/)
    src/main.rs
        Rôle : Point d'entrée exécutable du programme.
        
        Détails :
        
        Applique #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] pour masquer la fenêtre de terminal noire sous Windows lors d'une compilation en mode --release.
        
        Déclare l'arbre des modules du projet (app, ics, models, theme, ui).
        
        Configure le ViewportBuilder d'eframe : taille initiale (560 x 830 px), taille minimale (480 x 680 px), redimensionnement activé et mode "toujours au premier plan" (with_always_on_top()).
        
        Instancie DuckApp::new() et démarre la boucle d'événements native (eframe::run_native).
        
    src/app.rs
        Rôle : Chef d'orchestre central (DuckApp) reliant les données et l'interface graphique.
        
        Détails :
        
        MainTab : Énumération représentant les 3 écrans disponibles (Chronologie, Taches, Boutique).
        
        DuckApp::new() : Charge config.json, restaure save.json, vérifie si un nouveau jour a commencé, télécharge les cours et initialise le chronomètre avec la durée du jour.
        
        update_timer() : Décrémente timer_remaining chaque seconde lorsque le chronomètre de sport est en marche.
        
        daily_progress() : Compte le nombre total d'objectifs du jour affiché (cours + soin cheveux + sport) et combien ont été validés.
        
        eframe::App::update() : Appelée à chaque image (rafraîchie toutes les 100 ms), elle assemble dans l'ordre la barre du haut, le sélecteur de date, la grille Bento, l'onglet actif et la barre de navigation du bas.
        
    src/models.rs
        Rôle : Définition des structures de données et règles métier de gamification.
        
        Détails :
        
        AppConfig / HairDay / SportDay : Structures désérialisées depuis config.json grâce à serde.
        
        SaveData::load() & SaveData::save() : Gèrent la lecture sécurisée (avec valeurs par défaut si le fichier n'existe pas) et la sérialisation formatée de save.json.
        
        SaveData::check_new_day() : Compare last_date avec la date du jour ; si la date a changé, réinitialise les cases quotidiennes.
        
        SaveData::add_reward() : Ajoute l'XP et les graines gagnées, puis vérifie si le palier level * 100 est atteint pour monter de niveau (avec un bonus de +25 graines).
        
        SaveData::remove_reward() : Retire l'XP et les graines si une tâche est décochée par erreur, en gérant proprement la rétrogradation de niveau si nécessaire.
        
    src/ics.rs
        Rôle : Récupération, analyse syntaxique et filtrage de l'emploi du temps .ics.
        
        Détails :
        
        CourseEvent : Représente un cours avec son identifiant unique (uid), son intitulé (summary), sa salle (location) et ses horaires début/fin (NaiveDateTime).
        
        fetch_all_courses() : Télécharge le flux via ureq::get, met à jour schedule_cache.ics et déclenche l'analyse.
        
        courses_for_date() : Extrait uniquement les cours d'une journée donnée et les trie par heure de début.
        
        parse_ics() & should_ignore_course() : Déplie les lignes RFC 5545, convertit les dates UTC en heure locale (parse_ics_datetime) et applique les règles personnalisées :
        
        Masque Französisch 3 et Physique 2, complément tous les lundis entre 16h15 et 18h00.
        
        Renomme Communication Langue Etrangère en Anglais avec la salle VS-ENP.21.N405 (Classe).
        
    src/theme.rs
        Rôle : Charte graphique et utilitaires de mise en forme du texte.
        
        Détails :
        
        Palette : Regroupe toutes les constantes de couleurs Color32 (fonds sombres chauds, bordures, texte principal/atténué et couleurs d'accentuation vert, or, corail, bleu, violet).
        
        french_date_label() : Traduit une NaiveDate en format court français (ex. lun., 23 nov.).
        
        clean_course_label() : Nettoie les intitulés bruts de la HES-SO (supprime I3, P3 • HEI  et raccourcit VS-ENP.23.N319 en N319) tout en préservant l'affichage complet pour Anglais.
        
  3. Composants d'interface graphique (src/ui/)
    src/ui/mod.rs
        Rôle : Fichier index du module ui.
        
        Détails : Déclare et rend publics (pub mod) les trois sous-modules visuels (header, timeline, tabs) afin qu'ils soient accessibles depuis app.rs.
        
    src/ui/header.rs
        Rôle : Éléments visuels entourant la zone centrale (haut et bas de la fenêtre).
        
        Détails :
        
        render_top_bar() : Affiche le bouton de synchronisation manuelle (🔄) qui recharge l'URL .ics et le bouton épingle (📌) qui active/désactive le maintien au premier plan.
        
        render_date_selector() : Affiche le titre (Aujourd'hui ou Planning), la date formatée et les boutons < / > pour avancer ou reculer jour par jour.
        
        render_bento_stats() : Dessine la grille supérieure en deux colonnes : à gauche, une courbe vectorielle illustrant la progression d'XP vers le niveau suivant ; à droite, trois cartes (stat_row_card) pour le nombre de cours, les graines et la jauge circulaire (draw_ring_gauge) du taux de complétion.
        
        render_tab_switcher() : Affiche le badge doré du nombre de tâches restantes et le sélecteur rapide Chronologie / Routines.
        
        render_bottom_nav() : Barre de navigation inférieure à 3 colonnes (Chronologie, Tâches, Récompenses) avec indicateur coloré sous l'onglet actif.
        
    src/ui/timeline.rs
        Rôle : Affichage visuel de la journée sous forme de frise horaire.
        
        Détails :
        
        render_timeline() : Crée une zone défilante (ScrollArea) graduée de 07:00 à 22:00 avec des boutons + et - permettant de modifier dynamiquement l'échelle verticale (timeline_zoom).
        
        draw_timeline_block() : Dessine chaque événement sous forme de rectangle translucide bordé d'un liseré coloré à gauche (vert pour le soin cheveux, bleu pour les cours, or pour le sport).
        
        Trace une ligne horizontale corail accompagnée d'un badge horaire indiquant l'heure actuelle en temps réel lorsque la date sélectionnée est aujourd'hui.
        
    src/ui/tabs.rs
        Rôle : Contenu interactif des onglets secondaires.
        
        Détails :
        
        render_tasks_tab() : Affiche la liste interactive des objectifs du jour (soin cheveux, séance de sport, cours). Chaque case à cocher est bidirectionnelle : cocher crédite XP et graines via add_reward, décocher les retire via remove_reward. Intègre également les contrôles du chronomètre sportif (Start, Pause, Reset).
        
        render_rewards_tab() : Boutique de récompenses personnelles permettant d'échanger ses graines accumulées contre des pauses définies (café/Monster, session gaming, film, etc.).
        
🚀 Commandes utiles
    Bash
        # Compiler et lancer en mode développement
        cargo run
        
        # Compiler et lancer la version finale optimisée (sans console noire)
        cargo run --release
