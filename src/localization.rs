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

        en.insert("obj_destroy_boss", "- Destroy the Boss");
        ru.insert("obj_destroy_boss", "- Уничтожить босса");

        en.insert("press_space", "Press [SPACE] to Launch");
        ru.insert("press_space", "Нажми [ПРОБЕЛ] для запуска");

        // --- GAMEPLAY / UI ---
        en.insert("mission_complete", "MISSION COMPLETE!");
        ru.insert("mission_complete", "МИССИЯ ВЫПОЛНЕНА!");

        en.insert("level_cleared_prefix", "Level");
        ru.insert("level_cleared_prefix", "Уровень");

        en.insert("level_cleared_suffix", "Cleared");
        ru.insert("level_cleared_suffix", "Пройден");

        en.insert("next_mission", "Press [ENTER] for Upgrades");
        ru.insert("next_mission", "Нажми [ENTER] для улучшений");

        en.insert("game_over", "GAME OVER");
        ru.insert("game_over", "ИГРА ОКОНЧЕНА");

        en.insert("final_score_prefix", "Final Score:");
        ru.insert("final_score_prefix", "Итоговый счет:");

        en.insert("high_score", "HIGH SCORE:");
        ru.insert("high_score", "РЕКОРД:");

        en.insert("press_enter_resume", "Press [ENTER] to Resume");
        ru.insert("press_enter_resume", "Нажми [ENTER] для продолжения");

        en.insert("press_esc", "Press [ESC] for Main Menu");
        ru.insert("press_esc", "Нажми [ESC] для выхода в меню");

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

        en.insert("defeated", "Defeated:");
        ru.insert("defeated", "Побеждено:");

        en.insert("rust", "Rust:");
        ru.insert("rust", "Лом:");

        en.insert("gold", "Gold:");
        ru.insert("gold", "Золото:");

        en.insert("boss_hp", "Boss HP:");
        ru.insert("boss_hp", "Босс ЗД:");

        en.insert("boss_defeated", "Boss Defeated!");
        ru.insert("boss_defeated", "Босс повержен!");

        en.insert("resources", "Resources:");
        ru.insert("resources", "Ресурсы:");

        // --- UPGRADES ---
        en.insert("upgrade_bay_title", "UPGRADE BAY");
        ru.insert("upgrade_bay_title", "ОТСЕК УЛУЧШЕНИЙ");
        en.insert(
            "upgrade_bay_subtitle",
            "Spend resources before the next mission",
        );
        ru.insert(
            "upgrade_bay_subtitle",
            "Потрать ресурсы перед следующей миссией",
        );
        en.insert("upgrade_level", "Lvl");
        ru.insert("upgrade_level", "Ур");
        en.insert("upgrade_cost", "Cost");
        ru.insert("upgrade_cost", "Цена");
        en.insert("upgrade_status_maxed", "MAXED");
        ru.insert("upgrade_status_maxed", "МАКС");
        en.insert("upgrade_status_buy", "Ready");
        ru.insert("upgrade_status_buy", "Можно купить");
        en.insert("upgrade_status_lack", "Not enough");
        ru.insert("upgrade_status_lack", "Не хватает");
        en.insert("upgrade_continue", "Continue to Briefing");
        ru.insert("upgrade_continue", "Продолжить к брифингу");
        en.insert(
            "upgrade_controls",
            "UP/DOWN: Select  ENTER: Buy or Continue",
        );
        ru.insert(
            "upgrade_controls",
            "ВВЕРХ/ВНИЗ: Выбор  ENTER: Купить или продолжить",
        );
        en.insert("upgrade_hull_name", "Reinforced Hull");
        ru.insert("upgrade_hull_name", "Укрепленный корпус");
        en.insert("upgrade_hull_desc", "+20 max HP per level");
        ru.insert("upgrade_hull_desc", "+20 к макс. здоровью за уровень");
        en.insert("upgrade_weapon_name", "Weapon Tuning");
        ru.insert("upgrade_weapon_name", "Настройка оружия");
        en.insert("upgrade_weapon_desc", "+10% base bullet damage per level");
        ru.insert("upgrade_weapon_desc", "+10% к базовому урону за уровень");
        en.insert("upgrade_engine_name", "Engine Overdrive");
        ru.insert("upgrade_engine_name", "Форсаж двигателя");
        en.insert("upgrade_engine_desc", "+8% acceleration per level");
        ru.insert("upgrade_engine_desc", "+8% к ускорению за уровень");
        en.insert("upgrade_magnet_name", "Magnet Array");
        ru.insert("upgrade_magnet_name", "Магнитный массив");
        en.insert("upgrade_magnet_desc", "+25 loot magnet radius per level");
        ru.insert("upgrade_magnet_desc", "+25 к радиусу притяжения за уровень");
        en.insert("upgrade_shield_name", "Shield Capacitor");
        ru.insert("upgrade_shield_name", "Конденсатор щита");
        en.insert("upgrade_shield_desc", "Start mission with a small shield");
        ru.insert("upgrade_shield_desc", "Старт миссии с небольшим щитом");

        // --- MENU ITEMS ---
        en.insert("menu_start", "START");
        ru.insert("menu_start", "НАЧАТЬ");

        en.insert("menu_difficulty", "Difficulty");
        ru.insert("menu_difficulty", "Сложность");

        en.insert("menu_language", "Language");
        ru.insert("menu_language", "Язык");
        en.insert("menu_master_volume", "Master Volume");
        ru.insert("menu_master_volume", "Общая громкость");
        en.insert("menu_music_volume", "Music Volume");
        ru.insert("menu_music_volume", "Громкость музыки");
        en.insert("menu_sfx_volume", "SFX Volume");
        ru.insert("menu_sfx_volume", "Громкость эффектов");
        en.insert("menu_audio_mute", "Audio");
        ru.insert("menu_audio_mute", "Звук");
        en.insert("audio_on", "ON");
        ru.insert("audio_on", "ВКЛ");
        en.insert("audio_off", "OFF");
        ru.insert("audio_off", "ВЫКЛ");

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
        de.insert("obj_destroy_boss", "- Zerstöre den Boss");
        de.insert("press_space", "Drücke [LEERTASTE] zum Starten");
        de.insert("mission_complete", "MISSION ERFOLGREICH!");
        de.insert("level_cleared_prefix", "Level");
        de.insert("level_cleared_suffix", "Geschafft");
        de.insert("next_mission", "Drücke [ENTER] für Verbesserungen");
        de.insert("game_over", "SPIEL VORBEI");
        de.insert("final_score_prefix", "Endpunktzahl:");
        de.insert("high_score", "REKORD:");
        de.insert("press_enter_resume", "Drücke [ENTER] zum Fortsetzen");
        de.insert("press_esc", "Drücke [ESC] für Hauptmenü");
        de.insert("controls", "PFEILE zum Bewegen | LEERTASTE zum Schießen");
        de.insert("paused", "PAUSIERT");
        de.insert("score", "PUNKTE:");
        de.insert("hp", "LP:");
        de.insert("shield", "SCHILD:");
        de.insert("defeated", "Besiegt:");
        de.insert("rust", "Rost:");
        de.insert("gold", "Gold:");
        de.insert("boss_hp", "Boss LP:");
        de.insert("boss_defeated", "Boss besiegt!");
        de.insert("resources", "Ressourcen:");
        de.insert("upgrade_bay_title", "UPGRADE-BAU");
        de.insert(
            "upgrade_bay_subtitle",
            "Gib Ressourcen vor der nächsten Mission aus",
        );
        de.insert("upgrade_level", "Stufe");
        de.insert("upgrade_cost", "Kosten");
        de.insert("upgrade_status_maxed", "MAX");
        de.insert("upgrade_status_buy", "Kaufbar");
        de.insert("upgrade_status_lack", "Nicht genug");
        de.insert("upgrade_continue", "Zum Briefing weiter");
        de.insert(
            "upgrade_controls",
            "OBEN/UNTEN: Auswahl  ENTER: Kaufen oder weiter",
        );
        de.insert("upgrade_hull_name", "Verstärkte Hülle");
        de.insert("upgrade_hull_desc", "+20 max. LP pro Stufe");
        de.insert("upgrade_weapon_name", "Waffen-Tuning");
        de.insert("upgrade_weapon_desc", "+10% Grundschaden pro Stufe");
        de.insert("upgrade_engine_name", "Motor-Overdrive");
        de.insert("upgrade_engine_desc", "+8% Beschleunigung pro Stufe");
        de.insert("upgrade_magnet_name", "Magnet-Array");
        de.insert("upgrade_magnet_desc", "+25 Magnetradius pro Stufe");
        de.insert("upgrade_shield_name", "Schildkondensator");
        de.insert("upgrade_shield_desc", "Mission beginnt mit kleinem Schild");
        de.insert("menu_start", "STARTEN");
        de.insert("menu_difficulty", "Schwierigkeit");
        de.insert("menu_language", "Sprache");
        de.insert("menu_master_volume", "Gesamtlautstärke");
        de.insert("menu_music_volume", "Musiklautstärke");
        de.insert("menu_sfx_volume", "Effektlautstärke");
        de.insert("menu_audio_mute", "Audio");
        de.insert("audio_on", "AN");
        de.insert("audio_off", "AUS");
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
