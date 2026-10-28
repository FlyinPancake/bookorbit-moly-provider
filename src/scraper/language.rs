//! Book languages, as moly.hu writes them in Hungarian

/// Hungarian adjectives moly.hu uses for languages, in tags ("angol nyelvű") and in title
/// suffixes that tell apart editions with similar titles ("Dune (angol)").
const LANGUAGES: &[(&str, &str)] = &[
    ("magyar", "hu"),
    ("angol", "en"),
    ("német", "de"),
    ("francia", "fr"),
    ("olasz", "it"),
    ("spanyol", "es"),
    ("portugál", "pt"),
    ("orosz", "ru"),
    ("lengyel", "pl"),
    ("cseh", "cs"),
    ("szlovák", "sk"),
    ("román", "ro"),
    ("horvát", "hr"),
    ("szerb", "sr"),
    ("szlovén", "sl"),
    ("bolgár", "bg"),
    ("ukrán", "uk"),
    ("holland", "nl"),
    ("svéd", "sv"),
    ("norvég", "no"),
    ("dán", "da"),
    ("finn", "fi"),
    ("észt", "et"),
    ("lett", "lv"),
    ("litván", "lt"),
    ("görög", "el"),
    ("török", "tr"),
    ("héber", "he"),
    ("arab", "ar"),
    ("japán", "ja"),
    ("kínai", "zh"),
    ("koreai", "ko"),
    ("latin", "la"),
    ("eszperantó", "eo"),
    ("katalán", "ca"),
    ("ír", "ga"),
];

pub(super) fn language_code(adjective: &str) -> Option<&'static str> {
    let adjective = adjective.trim().to_lowercase();
    LANGUAGES
        .iter()
        .find(|(name, _)| *name == adjective)
        .map(|(_, code)| *code)
}

/// The language adjective of a tag such as "angol nyelvű"
pub(super) fn language_tag(tag: &str) -> Option<&str> {
    tag.strip_suffix(" nyelvű")
}

/// Splits "Dune (angol)" into "Dune" and "en". Other parentheses are part of the title.
pub(super) fn split_title_language(title: &str) -> (&str, Option<&'static str>) {
    title
        .strip_suffix(')')
        .and_then(|t| t.rsplit_once(" ("))
        .and_then(|(rest, lang)| Some((rest.trim_end(), Some(language_code(lang)?))))
        .unwrap_or((title, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_language_suffix() {
        assert_eq!(split_title_language("Dune (angol)"), ("Dune", Some("en")));
        assert_eq!(split_title_language("Dűne"), ("Dűne", None));
        // not a language, so it stays in the title
        assert_eq!(
            split_title_language("A Gyűrűk Ura (díszkiadás)"),
            ("A Gyűrűk Ura (díszkiadás)", None)
        );
    }
}
