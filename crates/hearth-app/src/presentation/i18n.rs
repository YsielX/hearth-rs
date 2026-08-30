use hearth_core::Locale;

/// Selects the localized copy for one of the runtime's supported locales.
pub fn pick<'a>(locale: Locale, en_us: &'a str, zh_cn: &'a str, zh_tw: &'a str) -> &'a str {
    match locale {
        Locale::EnUs => en_us,
        Locale::ZhCn => zh_cn,
        Locale::ZhTw => zh_tw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_picker_covers_all_supported_locales() {
        assert_eq!(pick(Locale::EnUs, "A", "简", "繁"), "A");
        assert_eq!(pick(Locale::ZhCn, "A", "简", "繁"), "简");
        assert_eq!(pick(Locale::ZhTw, "A", "简", "繁"), "繁");
    }
}
