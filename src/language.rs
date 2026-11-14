#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Swedish,
    English,
}
impl Language {
    pub fn text<'a>(self, swedish: &'a str, english: &'a str) -> &'a str {
        match self {
            Self::Swedish => swedish,
            Self::English => english,
        }
    }
    pub fn load() -> Self {
        let value = settings_path().and_then(|path| std::fs::read_to_string(path).ok());
        if value.as_deref().map(str::trim) == Some("en") {
            Self::English
        } else {
            Self::Swedish
        }
    }
    pub fn save(self) -> std::io::Result<()> {
        let path = settings_path()
            .ok_or_else(|| std::io::Error::other("Cannot locate settings folder"))?;
        std::fs::write(
            path,
            if self == Self::Swedish {
                "sv\n"
            } else {
                "en\n"
            },
        )
    }
}
fn settings_path() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(|folder| folder.join("NumberCatch.settings"))
}
