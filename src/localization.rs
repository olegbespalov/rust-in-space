use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Russian,
    German,
    #[default]
    #[serde(other)]
    English,
}

impl Language {
    pub fn from_system_locale() -> Self {
        Self::from_locale(sys_locale::get_locale().as_deref())
    }

    fn from_locale(locale: Option<&str>) -> Self {
        let language = locale
            .and_then(|locale| locale.split(['-', '_', '.', '@']).next())
            .unwrap_or_default();

        if language.eq_ignore_ascii_case("ru") {
            Self::Russian
        } else if language.eq_ignore_ascii_case("de") {
            Self::German
        } else {
            Self::English
        }
    }
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

        en.insert("goal_enemies", "Defeat enemy ships");
        ru.insert("goal_enemies", "Уничтожить вражеские корабли");

        en.insert("goal_asteroids", "Shoot down asteroids");
        ru.insert("goal_asteroids", "Расстрелять астероиды");

        en.insert("goal_interceptors", "Defeat interceptors");
        ru.insert("goal_interceptors", "Уничтожить перехватчики");

        en.insert("goal_heavy_ships", "Defeat heavy ships");
        ru.insert("goal_heavy_ships", "Уничтожить тяжёлые корабли");

        en.insert("goal_scrap", "Collect scrap");
        ru.insert("goal_scrap", "Собрать лом");

        en.insert("goal_survive", "Survive (sec)");
        ru.insert("goal_survive", "Продержаться (сек)");

        en.insert("obj_destroy_boss", "Defeat the Dreadnought");
        ru.insert("obj_destroy_boss", "Уничтожить Дредноут");

        en.insert("press_space", "Launch [SPACE]");
        ru.insert("press_space", "Запустить [ПРОБЕЛ]");

        // --- GAMEPLAY / UI ---
        en.insert("mission_complete", "MISSION COMPLETE!");
        ru.insert("mission_complete", "МИССИЯ ВЫПОЛНЕНА!");

        en.insert("level_cleared_prefix", "Level");
        ru.insert("level_cleared_prefix", "Уровень");

        en.insert("level_cleared_suffix", "Cleared");
        ru.insert("level_cleared_suffix", "Пройден");

        en.insert("next_mission", "Upgrades [ENTER]");
        ru.insert("next_mission", "Улучшения [ENTER]");

        en.insert("game_over", "GAME OVER");
        ru.insert("game_over", "ИГРА ОКОНЧЕНА");
        en.insert("return_to_menu", "Main Menu [ENTER]");
        ru.insert("return_to_menu", "Главное меню [ENTER]");

        en.insert("final_score_prefix", "Final Score:");
        ru.insert("final_score_prefix", "Итоговый счет:");

        en.insert("high_score", "HIGH SCORE:");
        ru.insert("high_score", "РЕКОРД:");

        en.insert("press_enter_resume", "Resume [ENTER]");
        ru.insert("press_enter_resume", "Продолжить [ENTER]");

        en.insert("press_esc", "Main Menu [ESC]");
        ru.insert("press_esc", "Главное меню [ESC]");

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
        en.insert("invisibility", "CLOAK:");
        ru.insert("invisibility", "МАСКИРОВКА:");

        en.insert("rust", "Rust:");
        ru.insert("rust", "Лом:");

        en.insert("gold", "Gold:");
        ru.insert("gold", "Золото:");

        en.insert("boss_hp", "Boss HP:");
        ru.insert("boss_hp", "Босс ЗД:");
        en.insert("boss_phase", "Phase");
        ru.insert("boss_phase", "Фаза");

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
        en.insert("player_name_prompt", "What's your name, Commander?");
        ru.insert("player_name_prompt", "Как вас зовут, командир?");
        en.insert("player_welcome", "Welcome");
        ru.insert("player_welcome", "Добро пожаловать");
        en.insert("commander_title", "Commander");
        ru.insert("commander_title", "Командир");
        en.insert("ship_selection", "Choose your ship");
        ru.insert("ship_selection", "Выбор корабля");
        en.insert("ship_alpha", "Alpha");
        ru.insert("ship_alpha", "Alpha");
        en.insert("ship_delta", "Delta");
        ru.insert("ship_delta", "Дельта");
        en.insert("ship_specter", "Specter");
        ru.insert("ship_specter", "Спектр");
        en.insert("ship_vector", "Vector");
        ru.insert("ship_vector", "Вектор");
        en.insert("ship_confirm", "Continue [Enter]");
        ru.insert("ship_confirm", "Далее [Enter]");
        en.insert("ship_back", "Back [Esc]");
        ru.insert("ship_back", "Назад [Esc]");

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
        de.insert("goal_enemies", "Feindschiffe zerstören");
        de.insert("goal_asteroids", "Asteroiden abschießen");
        de.insert("goal_interceptors", "Abfangjäger zerstören");
        de.insert("goal_heavy_ships", "Schwere Schiffe zerstören");
        de.insert("goal_scrap", "Schrott sammeln");
        de.insert("goal_survive", "Überleben (Sek.)");
        de.insert("obj_destroy_boss", "Den Dreadnought besiegen");
        de.insert("press_space", "Starten [LEERTASTE]");
        de.insert("mission_complete", "MISSION ERFOLGREICH!");
        de.insert("level_cleared_prefix", "Level");
        de.insert("level_cleared_suffix", "Geschafft");
        de.insert("next_mission", "Verbesserungen [ENTER]");
        de.insert("game_over", "SPIEL VORBEI");
        de.insert("return_to_menu", "Hauptmenü [ENTER]");
        de.insert("final_score_prefix", "Endpunktzahl:");
        de.insert("high_score", "REKORD:");
        de.insert("press_enter_resume", "Fortsetzen [ENTER]");
        de.insert("press_esc", "Hauptmenü [ESC]");
        de.insert("controls", "PFEILE zum Bewegen | LEERTASTE zum Schießen");
        de.insert("paused", "PAUSIERT");
        de.insert("score", "PUNKTE:");
        de.insert("hp", "LP:");
        de.insert("shield", "SCHILD:");
        de.insert("invisibility", "TARNUNG:");
        de.insert("rust", "Rost:");
        de.insert("gold", "Gold:");
        de.insert("boss_hp", "Boss LP:");
        de.insert("boss_phase", "Phase");
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
        de.insert("player_name_prompt", "Wie lautet dein Name, Commander?");
        de.insert("player_welcome", "Willkommen");
        de.insert("commander_title", "Commander");
        de.insert("ship_selection", "Raumschiff wählen");
        de.insert("ship_alpha", "Alpha");
        de.insert("ship_delta", "Delta");
        de.insert("ship_specter", "Specter");
        de.insert("ship_vector", "Vector");
        de.insert("ship_confirm", "Weiter [Enter]");
        de.insert("ship_back", "Zurück [Esc]");
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

        for (key, english, russian, german) in [
            (
                "lan_menu",
                "Local Network",
                "Локальная сеть",
                "Lokales Netzwerk",
            ),
            (
                "lan_host",
                "Host game [H]",
                "Создать игру [H]",
                "Spiel hosten [H]",
            ),
            (
                "lan_ip",
                "Host IPv4 address [Tab]",
                "IPv4-адрес хоста [Tab]",
                "IPv4-Adresse des Hosts [Tab]",
            ),
            (
                "lan_join",
                "Connect by IP",
                "Войти по IP",
                "Per IP verbinden",
            ),
            (
                "lan_games",
                "Available games",
                "Доступные игры",
                "Verfügbare Spiele",
            ),
            (
                "lan_searching",
                "Searching for games...",
                "Поиск игр...",
                "Suche nach Spielen...",
            ),
            (
                "lan_search_unavailable",
                "Search unavailable. Connect by IP.",
                "Поиск недоступен. Войдите по IP.",
                "Suche nicht verfügbar. Per IP verbinden.",
            ),
            (
                "lan_invalid_ip",
                "Enter a valid IPv4 address.",
                "Введите корректный IPv4-адрес.",
                "Gültige IPv4-Adresse eingeben.",
            ),
            (
                "lan_connecting",
                "Connecting...",
                "Подключение...",
                "Verbindung wird aufgebaut...",
            ),
            (
                "lan_port_busy",
                "TCP port 41616 is in use.",
                "Порт TCP 41616 занят.",
                "TCP-Port 41616 ist belegt.",
            ),
            (
                "lan_host_failed",
                "Could not create the game.",
                "Не удалось создать игру.",
                "Spiel konnte nicht erstellt werden.",
            ),
            (
                "lan_connect_failed",
                "Could not connect. Check the host address.",
                "Не удалось войти. Проверьте адрес хоста.",
                "Verbindung fehlgeschlagen. Host-Adresse prüfen.",
            ),
            (
                "lan_full",
                "Game is full or already started.",
                "Игра заполнена или уже началась.",
                "Spiel ist voll oder läuft bereits.",
            ),
            (
                "lan_version",
                "Incompatible game version.",
                "Несовместимая версия игры.",
                "Inkompatible Spielversion.",
            ),
            (
                "lan_host_lost",
                "Connection to the host lost.",
                "Связь с хостом потеряна.",
                "Verbindung zum Host verloren.",
            ),
            (
                "lan_guest_lost",
                "Guest disconnected.",
                "Гость отключился.",
                "Gast hat die Verbindung getrennt.",
            ),
            (
                "lan_left",
                "You left the session.",
                "Вы вышли из игры.",
                "Du hast die Sitzung verlassen.",
            ),
            ("lan_lobby", "LAN lobby", "Лобби LAN", "LAN-Lobby"),
            ("lan_slot_host", "Host", "Хост", "Host"),
            ("lan_slot_you", "You", "Вы", "Du"),
            ("lan_slot_empty", "Open slot", "Свободно", "Freier Platz"),
            (
                "lan_no_ip",
                "No local IPv4 address found.",
                "Локальный IPv4-адрес не найден.",
                "Keine lokale IPv4-Adresse gefunden.",
            ),
            (
                "lan_wait_guest",
                "Waiting for a guest...",
                "Ожидание гостя...",
                "Warte auf einen Gast...",
            ),
            (
                "lan_ready_count",
                "Guests ready",
                "Готовность гостей",
                "Gäste bereit",
            ),
            (
                "lan_start",
                "Start campaign [Enter]",
                "Начать кампанию [Enter]",
                "Kampagne starten [Enter]",
            ),
            (
                "lan_ready",
                "Ready [Enter]",
                "Готов [Enter]",
                "Bereit [Enter]",
            ),
            (
                "lan_not_ready",
                "Cancel ready [Enter]",
                "Отменить готовность",
                "Bereitschaft aufheben",
            ),
            (
                "lan_wait_launch",
                "Waiting for the host to launch...",
                "Хост запускает миссию...",
                "Warte auf den Missionsstart...",
            ),
            (
                "lan_wait_workshop",
                "Waiting for the host to open the Upgrade Bay...",
                "Ожидание входа хоста в мастерскую...",
                "Warte auf die Werkstatt...",
            ),
            (
                "lan_mvp",
                "Player of the Match",
                "Игрок матча",
                "Spieler des Matches",
            ),
            (
                "lan_mvp_points",
                "Contribution",
                "Очки вклада",
                "Beitragspunkte",
            ),
            (
                "lan_next_briefing",
                "Next briefing [Enter]",
                "К брифингу [Enter]",
                "Nächste Einweisung [Enter]",
            ),
            (
                "lan_ready_advance",
                "Ready for the next mission",
                "Можно начать следующую миссию",
                "Bereit für die nächste Mission",
            ),
            (
                "lan_wait_guest_ready",
                "Waiting for guests to be ready...",
                "Ожидание готовности гостей...",
                "Warte auf die Bereitschaft der Gäste...",
            ),
            (
                "lan_wait_briefing",
                "Waiting for the host to continue...",
                "Ожидание перехода к брифингу...",
                "Warte auf den Host...",
            ),
            (
                "lan_upgrade_ready",
                "Mark ready when finished.",
                "Отметьте готовность после покупок.",
                "Nach dem Kauf Bereitschaft melden.",
            ),
            ("lan_spectating", "Spectating", "Наблюдение", "Zuschauen"),
            (
                "lan_switch_pilot",
                "[Tab] Switch pilot",
                "[Tab] Сменить пилота",
                "[Tab] Pilot wechseln",
            ),
            (
                "lan_session_menu",
                "Session menu",
                "Меню сессии",
                "Sitzungsmenü",
            ),
            (
                "lan_resume",
                "Resume [Enter]",
                "Продолжить [Enter]",
                "Fortsetzen [Enter]",
            ),
            (
                "lan_leave",
                "Leave session",
                "Выйти из сессии",
                "Sitzung verlassen",
            ),
            (
                "lan_back_lobby",
                "Return to lobby [Enter]",
                "В лобби [Enter]",
                "Zur Lobby [Enter]",
            ),
            (
                "lan_wait_lobby",
                "Waiting for the host to return to the lobby...",
                "Ожидание возврата хоста в лобби...",
                "Warte auf die Rückkehr zur Lobby...",
            ),
        ] {
            en.insert(key, english);
            ru.insert(key, russian);
            de.insert(key, german);
        }

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

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn selects_supported_system_languages_and_falls_back_to_english() {
        for (locale, expected) in [
            (Some("ru_RU.UTF-8"), Language::Russian),
            (Some("de-AT"), Language::German),
            (Some("DE_DE@euro"), Language::German),
            (Some("en-GB"), Language::English),
            (Some("fr-FR"), Language::English),
            (Some("C.UTF-8"), Language::English),
            (None, Language::English),
        ] {
            assert_eq!(Language::from_locale(locale), expected, "{locale:?}");
        }
    }
}
