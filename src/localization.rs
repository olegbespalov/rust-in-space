use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Russian,
    German,
}

pub struct Localization {
    pub current_lang: Language,
    en_dict: HashMap<&'static str, &'static str>,
    ru_dict: HashMap<&'static str, &'static str>,
    de_dict: HashMap<&'static str, &'static str>,
}

impl Localization {
    pub fn new() -> Self {
        let mut en = HashMap::new();
        let mut ru = HashMap::new();
        let mut de = HashMap::new();

        // --- MENU ---
        en.insert("press_enter", "Press [ENTER] to Start");
        ru.insert("press_enter", "Нажми [ENTER] для старта");

        en.insert("difficulty", "DIFFICULTY (Left/Right Arrows):");
        ru.insert("difficulty", "СЛОЖНОСТЬ (Стрелки Влево/Вправо):");

        en.insert("diff_nebula", "NEBULA (Easy)");
        ru.insert("diff_nebula", "ТУМАННОСТЬ (Легко)");

        en.insert("diff_supernova", "SUPERNOVA (Normal)");
        ru.insert("diff_supernova", "СВЕРХНОВАЯ (Норма)");

        en.insert("diff_blackhole", "BLACK HOLE (Hard)");
        ru.insert("diff_blackhole", "ЧЕРНАЯ ДЫРА (Сложно)");

        en.insert("change_lang", "Press [L] to change Language");
        ru.insert("change_lang", "Нажми [L] для смены языка");

        // --- BRIEFING ---
        en.insert("mission", "MISSION");
        ru.insert("mission", "МИССИЯ");

        en.insert("objectives", "OBJECTIVES:");
        ru.insert("objectives", "ЦЕЛИ:");

        en.insert("obj_destroy_prefix", "- Destroy");
        ru.insert("obj_destroy_prefix", "- Уничтожить");

        en.insert("obj_scrap_prefix", "- Collect");
        ru.insert("obj_scrap_prefix", "- Собрать");

        en.insert("obj_gold_prefix", "- Collect");
        ru.insert("obj_gold_prefix", "- Собрать");

        en.insert("obj_enemies", "Enemies");
        ru.insert("obj_enemies", "Врагов");

        en.insert("obj_rust_piles", "Rust Piles");
        ru.insert("obj_rust_piles", "Куч Лома");

        en.insert("obj_gold", "Gold");
        ru.insert("obj_gold", "Золота");

        en.insert("press_space", "Press [SPACE] to Launch");
        ru.insert("press_space", "Нажми [ПРОБЕЛ] для запуска");

        // --- GAMEPLAY / UI ---
        en.insert("mission_complete", "MISSION COMPLETE!");
        ru.insert("mission_complete", "МИССИЯ ВЫПОЛНЕНА!");

        en.insert("level_cleared_prefix", "Level");
        ru.insert("level_cleared_prefix", "Уровень");

        en.insert("level_cleared_suffix", "Cleared");
        ru.insert("level_cleared_suffix", "Пройден");

        en.insert("next_mission", "Press [ENTER] for Next Mission");
        ru.insert("next_mission", "Нажми [ENTER] для след. миссии");

        en.insert("game_over", "GAME OVER");
        ru.insert("game_over", "ИГРА ОКОНЧЕНА");

        en.insert("final_score_prefix", "Final Score:");
        ru.insert("final_score_prefix", "Итоговый счет:");

        en.insert("high_score", "HIGH SCORE:");
        ru.insert("high_score", "РЕКОРД:");

        en.insert("press_esc", "Press [ESC] to Resume");
        ru.insert("press_esc", "Нажми [ESC] для продолжения");

        en.insert("controls", "ARROWS to move | SPACE to shoot");
        ru.insert("controls", "СТРЕЛКИ для движения | ПРОБЕЛ для стрельбы");

        en.insert("paused", "PAUSED");
        ru.insert("paused", "ПАУЗА");

        // --- IN-GAME UI ---
        en.insert("score", "SCORE:");
        ru.insert("score", "СЧЕТ:");

        en.insert("hp", "HP:");
        ru.insert("hp", "ЗДОРОВЬЕ:");

        en.insert("shield", "SHIELD:");
        ru.insert("shield", "ЩИТ:");

        en.insert("kills", "Kills:");
        ru.insert("kills", "Убийств:");

        en.insert("rust", "Rust:");
        ru.insert("rust", "Лом:");

        en.insert("gold", "Gold:");
        ru.insert("gold", "Золото:");

        en.insert("resources", "Resources:");
        ru.insert("resources", "Ресурсы:");

        // --- MENU ITEMS ---
        en.insert("menu_start", "START");
        ru.insert("menu_start", "НАЧАТЬ");

        en.insert("menu_difficulty", "Difficulty");
        ru.insert("menu_difficulty", "Сложность");

        en.insert("menu_language", "Language");
        ru.insert("menu_language", "Язык");

        en.insert("lang_english", "English");
        ru.insert("lang_english", "Английский");

        en.insert("lang_russian", "Russian");
        ru.insert("lang_russian", "Русский");

        en.insert(
            "menu_instructions",
            "UP/DOWN: Select  LEFT/RIGHT: Change  ENTER: Confirm",
        );
        ru.insert(
            "menu_instructions",
            "ВВЕРХ/ВНИЗ: Выбрать  ВЛЕВО/ВПРАВО: Изменить  ENTER: Подтвердить",
        );
        de.insert(
            "menu_instructions",
            "OBEN/UNTEN: Auswählen  LINKS/RECHTS: Ändern  ENTER: Bestätigen",
        );

        // German translations
        de.insert("press_enter", "Drücke [ENTER] zum Starten");
        de.insert("difficulty", "SCHWIERIGKEIT (Links/Rechts Pfeile):");
        de.insert("diff_nebula", "NEBEL (Einfach)");
        de.insert("diff_supernova", "SUPERNOVA (Normal)");
        de.insert("diff_blackhole", "SCHWARZES LOCH (Schwer)");
        de.insert("change_lang", "Drücke [L] um Sprache zu ändern");
        de.insert("mission", "MISSION");
        de.insert("objectives", "ZIELE:");
        de.insert("obj_destroy_prefix", "- Zerstöre");
        de.insert("obj_scrap_prefix", "- Sammle");
        de.insert("obj_gold_prefix", "- Sammle");
        de.insert("obj_enemies", "Feinde");
        de.insert("obj_rust_piles", "Rosthaufen");
        de.insert("obj_gold", "Gold");
        de.insert("press_space", "Drücke [LEERTASTE] zum Starten");
        de.insert("mission_complete", "MISSION ERFOLGREICH!");
        de.insert("level_cleared_prefix", "Level");
        de.insert("level_cleared_suffix", "Geschafft");
        de.insert("next_mission", "Drücke [ENTER] für nächste Mission");
        de.insert("game_over", "SPIEL VORBEI");
        de.insert("final_score_prefix", "Endpunktzahl:");
        de.insert("high_score", "REKORD:");
        de.insert("press_esc", "Drücke [ESC] zum Fortsetzen");
        de.insert("controls", "PFEILE zum Bewegen | LEERTASTE zum Schießen");
        de.insert("paused", "PAUSIERT");
        de.insert("score", "PUNKTE:");
        de.insert("hp", "LP:");
        de.insert("shield", "SCHILD:");
        de.insert("kills", "Kills:");
        de.insert("rust", "Rost:");
        de.insert("gold", "Gold:");
        de.insert("resources", "Ressourcen:");
        de.insert("menu_start", "STARTEN");
        de.insert("menu_difficulty", "Schwierigkeit");
        de.insert("menu_language", "Sprache");
        de.insert("lang_english", "Englisch");
        de.insert("lang_russian", "Russisch");
        de.insert("lang_german", "Deutsch");

        Self {
            current_lang: Language::English,
            en_dict: en,
            ru_dict: ru,
            de_dict: de,
        }
    }

    pub fn t(&self, key: &str) -> &str {
        let dict = match self.current_lang {
            Language::English => &self.en_dict,
            Language::Russian => &self.ru_dict,
            Language::German => &self.de_dict,
        };

        dict.get(key).unwrap_or(&"MISSING_TEXT")
    }

    pub fn cycle_lang(&mut self) {
        self.current_lang = match self.current_lang {
            Language::English => Language::Russian,
            Language::Russian => Language::German,
            Language::German => Language::English,
        };
    }
}
