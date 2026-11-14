use crate::{
    game::{Game, Mode},
    language::Language,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Settings,
    Playing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Fullscreen,
    Quit,
    SaveLanguage,
}
pub struct Application {
    pub screen: Screen,
    pub language: Language,
    pub game: Game,
    pub settings_error: bool,
}
impl Application {
    pub fn new(language: Language) -> Self {
        Self {
            screen: Screen::Menu,
            language,
            game: Game::for_mode(Mode::Math, language),
            settings_error: false,
        }
    }
    pub fn start(&mut self, mode: Mode) {
        self.game = Game::for_mode(mode, self.language);
        self.screen = Screen::Playing;
    }
    pub fn menu(&mut self) {
        self.game.paused = true;
        self.game.clear_input();
        self.screen = Screen::Menu;
    }
    pub fn hide_pointer(&self) -> bool {
        self.screen == Screen::Playing && !self.game.paused && !self.game.rules.game_over
    }
    pub fn update(&mut self, dt: f32) {
        if self.screen == Screen::Playing {
            self.game.update(dt);
        } else {
            self.game.animation += dt;
        }
    }
    fn choose_language(&mut self, language: Language) -> Action {
        self.language = language;
        self.game = Game::for_mode(self.game.rules.mode, language);
        self.settings_error = false;
        Action::SaveLanguage
    }
    pub fn key(&mut self, key: u32) -> Action {
        if key == 122 {
            return Action::Fullscreen;
        }
        match self.screen {
            Screen::Menu => match key {
                49 | 97 | 13 => self.start(Mode::Math),
                50 | 98 => self.start(Mode::Reading),
                83 => self.screen = Screen::Settings,
                27 | 81 => return Action::Quit,
                _ => {}
            },
            Screen::Settings => match key {
                83 | 37 => return self.choose_language(Language::Swedish),
                69 | 39 => return self.choose_language(Language::English),
                27 | 66 => self.screen = Screen::Menu,
                _ => {}
            },
            Screen::Playing => match key {
                32 if !self.game.rules.game_over => self.game.paused = !self.game.paused,
                13 if self.game.rules.game_over => self.start(self.game.rules.mode),
                82 => self.start(self.game.rules.mode),
                27 | 77 => self.menu(),
                _ => {}
            },
        }
        Action::None
    }
    pub fn click(&mut self, x: f32, y: f32) -> Action {
        let hit = |left: f32, top: f32, width: f32, height: f32| {
            x >= left && x <= left + width && y >= top && y <= top + height
        };
        match self.screen {
            Screen::Menu => {
                if hit(220.0, 220.0, 265.0, 220.0) {
                    self.start(Mode::Math);
                } else if hit(515.0, 220.0, 265.0, 220.0) {
                    self.start(Mode::Reading);
                } else if hit(300.0, 480.0, 400.0, 62.0) {
                    self.screen = Screen::Settings;
                } else if hit(350.0, 570.0, 300.0, 50.0) {
                    return Action::Quit;
                }
            }
            Screen::Settings => {
                if hit(220.0, 285.0, 265.0, 120.0) {
                    return self.choose_language(Language::Swedish);
                } else if hit(515.0, 285.0, 265.0, 120.0) {
                    return self.choose_language(Language::English);
                } else if hit(350.0, 500.0, 300.0, 65.0) {
                    self.screen = Screen::Menu;
                }
            }
            Screen::Playing => {
                if self.game.rules.game_over && hit(175.0, 440.0, 175.0, 50.0) {
                    self.start(self.game.rules.mode);
                } else if self.game.rules.game_over && hit(370.0, 440.0, 175.0, 50.0) {
                    self.menu();
                } else if hit(25.0, 100.0, 140.0, 34.0) {
                    self.menu();
                }
            }
        }
        Action::None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_in_swedish_menu() {
        let app = Application::new(Language::Swedish);
        assert_eq!(app.screen, Screen::Menu);
        assert_eq!(app.language, Language::Swedish);
    }
    #[test]
    fn mode_buttons_and_keyboard_work() {
        let mut app = Application::new(Language::Swedish);
        app.click(400.0, 265.0);
        assert_eq!(app.game.rules.mode, Mode::Math);
        app.key(27);
        assert_eq!(app.screen, Screen::Menu);
        app.click(650.0, 320.0);
        assert_eq!(app.game.rules.mode, Mode::Reading);
        assert_eq!(app.screen, Screen::Playing);
        app.key(27);
        app.key(49);
        assert_eq!(app.game.rules.mode, Mode::Math);
        app.key(27);
        app.key(50);
        assert_eq!(app.game.rules.mode, Mode::Reading);
    }
    #[test]
    fn settings_changes_both_modes() {
        let mut app = Application::new(Language::Swedish);
        app.key(83);
        assert_eq!(app.screen, Screen::Settings);
        assert_eq!(app.click(600.0, 340.0), Action::SaveLanguage);
        assert_eq!(app.language, Language::English);
        app.key(27);
        app.key(49);
        assert_eq!(app.game.rules.language, Language::English);
        app.menu();
        app.key(50);
        assert_eq!(app.game.rules.language, Language::English);
    }
    #[test]
    fn menus_do_not_run_gameplay() {
        let mut app = Application::new(Language::Swedish);
        app.update(1.0);
        assert_eq!(app.game.spawn_in, 1.2);
        assert!(app.game.drops.is_empty());
    }
    #[test]
    fn pointer_is_visible_in_menus_pause_and_game_over() {
        let mut app = Application::new(Language::Swedish);
        assert!(!app.hide_pointer());
        app.start(Mode::Math);
        assert!(app.hide_pointer());
        app.key(32);
        assert!(!app.hide_pointer());
        app.key(32);
        assert!(app.hide_pointer());
        app.game.rules.game_over = true;
        app.game.rules.growth = 0;
        assert!(!app.hide_pointer());
        app.key(13);
        assert!(!app.game.rules.game_over);
        assert_eq!(app.game.rules.growth, 2);
        assert!(app.hide_pointer());
        app.game.rules.game_over = true;
        app.click(450.0, 465.0);
        assert_eq!(app.screen, Screen::Menu);
    }
}
