use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Russian,
    German,
    Spanish,
    Japanese,
    Chinese,
    Korean,
    #[default]
    #[serde(other)]
    English,
}

const CATALOGS: &[(Language, &str, &str)] = &[
    (
        Language::English,
        "en",
        include_str!("../assets/locales/en.ftl"),
    ),
    (
        Language::Russian,
        "ru",
        include_str!("../assets/locales/ru.ftl"),
    ),
    (
        Language::German,
        "de",
        include_str!("../assets/locales/de.ftl"),
    ),
    (
        Language::Spanish,
        "es",
        include_str!("../assets/locales/es.ftl"),
    ),
    (
        Language::Japanese,
        "ja",
        include_str!("../assets/locales/ja.ftl"),
    ),
    (
        Language::Chinese,
        "zh-Hans",
        include_str!("../assets/locales/zh-Hans.ftl"),
    ),
    (
        Language::Korean,
        "ko",
        include_str!("../assets/locales/ko.ftl"),
    ),
];

impl Language {
    pub fn from_system_locale() -> Self {
        Self::from_locale(sys_locale::get_locale().as_deref())
    }

    fn from_locale(locale: Option<&str>) -> Self {
        let language = locale
            .and_then(|locale| locale.split(['-', '_', '.', '@']).next())
            .unwrap_or_default();

        CATALOGS
            .iter()
            .find(|(_, tag, _)| {
                tag.split('-')
                    .next()
                    .is_some_and(|base| base.eq_ignore_ascii_case(language))
            })
            .map_or(Self::English, |&(language, _, _)| language)
    }
}

pub struct Localization {
    pub current_lang: Language,
    bundles: HashMap<Language, FluentBundle<FluentResource>>,
}

impl Localization {
    pub fn new() -> Self {
        let bundles = CATALOGS
            .iter()
            .map(|&(language, tag, source)| {
                let locale = tag.parse().expect("valid bundled language tag");
                let resource =
                    FluentResource::try_new(source.to_owned()).unwrap_or_else(|(_, errors)| {
                        panic!("Invalid Fluent catalog {tag}: {errors:?}")
                    });
                let mut bundle = FluentBundle::new(vec![locale]);
                // Macroquad renders text without bidirectional layout.
                bundle.set_use_isolating(false);
                bundle
                    .add_resource(resource)
                    .unwrap_or_else(|errors| panic!("Invalid Fluent catalog {tag}: {errors:?}"));
                (language, bundle)
            })
            .collect();

        Self {
            current_lang: Language::English,
            bundles,
        }
    }

    pub fn t(&self, key: &str) -> Cow<'_, str> {
        self.format(key, None)
    }

    pub fn format(&self, key: &str, args: Option<&FluentArgs<'_>>) -> Cow<'_, str> {
        self.resolve(self.current_lang, key, args)
            .or_else(|| self.resolve(Language::English, key, args))
            .unwrap_or(Cow::Borrowed("MISSING_TEXT"))
    }

    fn resolve(
        &self,
        language: Language,
        key: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Option<Cow<'_, str>> {
        let bundle = self.bundles.get(&language)?;
        let message = bundle.get_message(key)?;
        let mut errors = Vec::new();
        let text = bundle.format_pattern(message.value()?, args, &mut errors);
        errors.is_empty().then_some(text)
    }

    pub fn cycle_lang(&mut self, backwards: bool) {
        let current = CATALOGS
            .iter()
            .position(|&(language, _, _)| language == self.current_lang)
            .expect("registered language");
        let step = if backwards { CATALOGS.len() - 1 } else { 1 };
        self.current_lang = CATALOGS[(current + step) % CATALOGS.len()].0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_supported_system_languages_and_falls_back_to_english() {
        for (locale, expected) in [
            (Some("ru_RU.UTF-8"), Language::Russian),
            (Some("de-AT"), Language::German),
            (Some("DE_DE@euro"), Language::German),
            (Some("es-419"), Language::Spanish),
            (Some("ja_JP.UTF-8"), Language::Japanese),
            (Some("zh-Hans-CN"), Language::Chinese),
            (Some("ko-KR"), Language::Korean),
            (Some("en-GB"), Language::English),
            (Some("fr-FR"), Language::English),
            (Some("C.UTF-8"), Language::English),
            (None, Language::English),
        ] {
            assert_eq!(Language::from_locale(locale), expected, "{locale:?}");
        }
    }

    #[test]
    fn every_catalog_has_the_same_messages_and_formats_their_arguments() {
        let localization = Localization::new();
        let mut args = FluentArgs::new();
        args.set("level", 12);
        args.set("name", "Pilot");

        for &(language, tag, source) in CATALOGS {
            let english = CATALOGS[0].2;
            let ids = |source: &'static str| -> std::collections::BTreeSet<&str> {
                source
                    .lines()
                    .filter(|line| line.starts_with(|c: char| c.is_ascii_alphabetic()))
                    .filter_map(|line| line.split_once(" =").map(|(key, _)| key))
                    .collect()
            };
            assert_eq!(ids(source), ids(english), "{tag}");
            for key in ids(english) {
                let text = localization
                    .resolve(language, key, Some(&args))
                    .unwrap_or_else(|| panic!("{tag}: cannot format {key}"));
                assert!(!text.is_empty(), "{tag}: empty message {key}");
                if key == "level_cleared" || key == "mission_deep_space" {
                    assert!(text.contains("12"), "{tag}: {key} omitted the level");
                }
                if key == "player_welcome" || key == "commander_title" {
                    assert!(text.contains("Pilot"), "{tag}: {key} omitted the name");
                }
            }
        }
    }

    #[test]
    fn cycles_languages_in_both_directions_and_restores_saved_values() {
        let mut localization = Localization::new();
        for &(language, _, _) in CATALOGS {
            assert_eq!(localization.current_lang, language);
            let saved = serde_json::to_string(&language).unwrap();
            assert_eq!(serde_json::from_str::<Language>(&saved).unwrap(), language);
            localization.cycle_lang(false);
        }
        assert_eq!(localization.current_lang, Language::English);
        localization.cycle_lang(true);
        assert_eq!(localization.current_lang, Language::Korean);
    }

    #[test]
    fn falls_back_to_english_for_a_missing_message() {
        let mut localization = Localization::new();
        localization.current_lang = Language::Japanese;
        localization.bundles.remove(&Language::Japanese);
        assert_eq!(localization.t("menu_start"), "START");
    }
}
