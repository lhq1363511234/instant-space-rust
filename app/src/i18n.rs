use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Zh,
    En,
}

impl Locale {
    pub fn from_code(code: &str) -> Self {
        if code.trim().to_ascii_lowercase().starts_with("zh") {
            Self::Zh
        } else {
            Self::En
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Zh => "zh-CN",
            Self::En => "en",
        }
    }
}

#[derive(Clone, Copy)]
pub struct I18n {
    pub locale: RwSignal<Locale>,
}

pub fn provide_i18n() -> I18n {
    let i18n = I18n {
        locale: RwSignal::new(default_locale()),
    };
    provide_context(i18n);
    i18n
}

pub fn use_i18n() -> I18n {
    use_context::<I18n>().expect("i18n context should be provided by App")
}

pub fn t(locale: Locale, zh: &'static str, en: &'static str) -> &'static str {
    match locale {
        Locale::Zh => zh,
        Locale::En => en,
    }
}

pub fn localize_optional(locale: Locale, zh: &str, en: Option<&str>) -> String {
    match locale {
        Locale::Zh => zh.to_string(),
        Locale::En => en
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(zh)
            .to_string(),
    }
}

pub fn localized_space_count(locale: Locale, count: usize) -> String {
    match locale {
        Locale::Zh => format!("{count} 个空间"),
        Locale::En => format!("{count} spaces"),
    }
}

pub fn localized_online_count(locale: Locale, count: i32) -> String {
    match locale {
        Locale::Zh => format!("{count} 人在线"),
        Locale::En => format!("{count} online"),
    }
}

fn default_locale() -> Locale {
    Locale::from_code(option_env!("INSTANT_SPACE_DEFAULT_LOCALE").unwrap_or("zh-CN"))
}
