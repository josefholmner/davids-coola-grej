use crate::language::Language;
use crate::vocabulary::{self, WORDS};
use std::time::{SystemTime, UNIX_EPOCH};
pub const FULL: i32 = 6;
pub const WIDTH: f32 = 1000.0;
pub const HEIGHT: f32 = 700.0;
pub const PLAY_RIGHT: f32 = 720.0;
pub const BASKET_Y: f32 = 590.0;
pub const BASKET_HALF: f32 = 56.0;

pub struct Rng(u64);
impl Rng {
    pub fn seeded(seed: u64) -> Self {
        Self(seed.max(1))
    }
    pub fn new() -> Self {
        Self::seeded(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        )
    }
    pub fn fraction(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        ((self.0 >> 40) as f32) / 16_777_216.0
    }
    pub fn integer(&mut self, low: i32, high: i32) -> i32 {
        debug_assert!(low <= high);
        low + (self.fraction() * (high - low + 1) as f32) as i32
    }
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let other = self.index(index + 1);
            values.swap(index, other);
        }
    }
    pub fn index(&mut self, length: usize) -> usize {
        self.integer(0, length as i32 - 1) as usize
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Math,
    Reading,
}
#[derive(Clone, Copy)]
pub struct Difficulty(pub u32);
impl Difficulty {
    pub fn subtraction(self) -> bool {
        self.0 % 3 == 0
    }
    pub fn max_operand(self) -> i32 {
        if self.subtraction() {
            (3 + (self.0 / 3 - 1) as i32).min(999)
        } else {
            (3 + (((self.0 - 1) * 2 + 1) / 3) as i32).min(9999)
        }
    }
    pub fn operand_limit(self, target: i32) -> i32 {
        if self.subtraction() {
            self.max_operand()
        } else {
            self.max_operand().max(target - 1)
        }
    }
    pub fn min_target(self) -> i32 {
        if self.subtraction() {
            1
        } else {
            (self.max_operand() - 1).max(2)
        }
    }
    pub fn max_target(self) -> i32 {
        if self.subtraction() {
            self.max_operand() - 1
        } else {
            self.max_operand() * 2
        }
    }
    pub fn speed(self) -> f32 {
        if self.subtraction() {
            (50.0 + (self.0 / 3 - 1) as f32 * 4.0).min(180.0)
        } else {
            (65.0 + (self.0 - 1) as f32 * 5.0).min(280.0)
        }
    }
    pub fn interval(self) -> f32 {
        if self.subtraction() {
            (2.2 - (self.0 / 3 - 1) as f32 * 0.04).max(1.2)
        } else {
            (1.9 - (self.0 - 1) as f32 * 0.035).max(0.9)
        }
    }
    pub fn reading_speed(self) -> f32 {
        (57.0 + (self.0 - 1) as f32 * 4.0).min(200.0)
    }
    pub fn reading_interval(self) -> f32 {
        (2.0 - (self.0 - 1) as f32 * 0.035).max(1.0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Add,
    Subtract,
}
#[derive(Clone, Copy, Debug)]
pub struct Problem {
    pub left: i32,
    pub right: i32,
    pub op: Op,
}
impl Problem {
    pub fn answer(self) -> i32 {
        match self.op {
            Op::Add => self.left + self.right,
            Op::Subtract => self.left - self.right,
        }
    }
    pub fn label(self) -> String {
        format!(
            "{} {} {}",
            self.left,
            if self.op == Op::Add { "+" } else { "−" },
            self.right
        )
    }
}
pub fn make_problem(target: i32, d: Difficulty, rng: &mut Rng) -> Problem {
    let matching = rng.fraction() < 0.5;
    make_problem_with_match(target, d, rng, matching)
}
fn make_problem_with_match(target: i32, d: Difficulty, rng: &mut Rng, matching: bool) -> Problem {
    let cap = d.operand_limit(target);
    if d.subtraction() {
        let mut answer = target;
        if !matching {
            answer = rng.integer(1, cap - 1);
            if answer == target {
                answer = if answer == cap - 1 { 1 } else { answer + 1 };
            }
        }
        let right = rng.integer(1, cap - answer);
        Problem {
            left: answer + right,
            right,
            op: Op::Subtract,
        }
    } else if matching {
        let left = rng.integer(1.max(target - cap), cap.min(target - 1));
        Problem {
            left,
            right: target - left,
            op: Op::Add,
        }
    } else {
        let left = rng.integer(1, cap);
        let mut right = rng.integer(1, cap);
        if left + right == target {
            right = right % cap + 1;
        }
        Problem {
            left,
            right,
            op: Op::Add,
        }
    }
}
// Small shuffled bags keep the mix fair while preventing long accidental runs.
#[derive(Default)]
struct MatchBag {
    remaining: Vec<bool>,
    last: Option<bool>,
    streak: usize,
}
impl MatchBag {
    fn observe(&mut self, value: bool) {
        self.streak = if self.last == Some(value) {
            self.streak + 1
        } else {
            1
        };
        self.last = Some(value);
    }
    fn after_first(value: bool, rng: &mut Rng) -> Self {
        let mut remaining = vec![true, false, !value];
        rng.shuffle(&mut remaining);
        Self {
            remaining,
            last: Some(value),
            streak: 1,
        }
    }
    fn next(&mut self, rng: &mut Rng) -> bool {
        if self.remaining.is_empty() {
            self.remaining = vec![true, true, false, false];
            rng.shuffle(&mut self.remaining);
        }
        let end = self.remaining.len() - 1;
        if self.streak >= 2 && self.last == Some(self.remaining[end]) {
            if let Some(other) = self
                .remaining
                .iter()
                .position(|value| Some(*value) != self.last)
            {
                self.remaining.swap(other, end);
            }
        }
        let value = self.remaining.pop().unwrap();
        self.observe(value);
        value
    }
}

#[derive(Clone, Copy)]
pub enum Content {
    Math(Problem),
    Word(usize),
}
impl Content {
    pub fn label(self, language: Language) -> String {
        match self {
            Self::Math(problem) => problem.label(),
            Self::Word(index) => WORDS[index].text(language).to_string(),
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Correct,
    Wrong,
    LevelUp,
    LevelDown,
}
pub struct Rules {
    pub mode: Mode,
    pub language: Language,
    pub level: u32,
    pub growth: i32,
    pub target: i32,
    pub target_word: usize,
    pub score: u64,
    pub last_answer: String,
}
impl Rules {
    #[cfg(test)]
    pub fn new(rng: &mut Rng) -> Self {
        Self::for_mode(Mode::Math, Language::Swedish, rng)
    }
    pub fn for_mode(mode: Mode, language: Language, rng: &mut Rng) -> Self {
        let mut state = Self {
            mode,
            language,
            level: 1,
            growth: 2,
            target: 0,
            target_word: 0,
            score: 0,
            last_answer: String::new(),
        };
        state.new_target(rng);
        state
    }
    pub fn difficulty(&self) -> Difficulty {
        Difficulty(self.level)
    }
    pub fn new_target(&mut self, rng: &mut Rng) {
        if self.mode == Mode::Reading {
            let pool = vocabulary::pool(self.level, self.language);
            let mut selected = rng.index(pool.len());
            if pool[selected] == self.target_word {
                selected = (selected + 1) % pool.len();
            }
            self.target_word = pool[selected];
            return;
        }
        let d = self.difficulty();
        let mut target = rng.integer(d.min_target(), d.max_target());
        if target == self.target {
            target = if target == d.max_target() {
                d.min_target()
            } else {
                target + 1
            };
        }
        self.target = target;
    }
    pub fn catch(&mut self, problem: Problem, rng: &mut Rng) -> Outcome {
        let answer = problem.answer();
        self.last_answer = format!("{} = {}", problem.label(), answer);
        self.apply_match(answer == self.target, self.target as u64 * 10, rng)
    }
    pub fn catch_word(&mut self, index: usize, rng: &mut Rng) -> Outcome {
        self.last_answer = WORDS[index].text(self.language).into();
        self.apply_match(
            index == self.target_word,
            WORDS[self.target_word].text(self.language).chars().count() as u64 * 10,
            rng,
        )
    }
    fn apply_match(&mut self, correct: bool, points: u64, rng: &mut Rng) -> Outcome {
        if correct {
            self.score += points;
            self.growth += 1;
            let result = if self.growth >= FULL {
                self.level += 1;
                self.growth = 2;
                Outcome::LevelUp
            } else {
                Outcome::Correct
            };
            self.new_target(rng);
            return result;
        }
        self.growth -= 1;
        if self.growth <= 0 {
            if self.level > 1 {
                self.level -= 1;
                self.growth = 3;
                self.new_target(rng);
                return Outcome::LevelDown;
            }
            self.growth = 1;
        }
        Outcome::Wrong
    }
}
pub struct Drop {
    pub x: f32,
    pub y: f32,
    pub half_width: f32,
    pub speed: f32,
    pub content: Content,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackKind {
    Correct,
    Wrong,
}
pub struct Feedback {
    pub kind: FeedbackKind,
    pub x: f32,
    pub age: f32,
}
pub struct Game {
    pub rules: Rules,
    pub rng: Rng,
    pub basket_x: f32,
    pub drops: Vec<Drop>,
    pub left: bool,
    pub right: bool,
    pub mouse_x: Option<f32>,
    pub paused: bool,
    pub display_growth: f32,
    pub animation: f32,
    pub transition: f32,
    pub spawn_in: f32,
    pub message: String,
    pub message_time: f32,
    pub feedback: Option<Feedback>,
    pub pending_sound: Option<FeedbackKind>,
    opening_order: MatchBag,
    match_order: MatchBag,
    round_key: Option<(Mode, Language, u32, i32)>,
    round_spawns: usize,
    math_choices: Vec<Problem>,
    last_math_choice: Option<Problem>,
    word_choices: Vec<usize>,
    last_wrong_word: Option<usize>,
}
impl Game {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::for_mode(Mode::Math, Language::Swedish)
    }
    pub fn for_mode(mode: Mode, language: Language) -> Self {
        let mut rng = Rng::new();
        let rules = Rules::for_mode(mode, language, &mut rng);
        let message = language
            .text(
                if mode == Mode::Math {
                    "Fånga rätt svar så växer ditt träd!"
                } else {
                    "Fånga ordet som passar bilden!"
                },
                if mode == Mode::Math {
                    "Catch matching answers to grow your tree!"
                } else {
                    "Catch the word that matches the picture!"
                },
            )
            .into();
        Self {
            rules,
            rng,
            basket_x: PLAY_RIGHT / 2.0,
            drops: Vec::new(),
            left: false,
            right: false,
            mouse_x: None,
            paused: false,
            display_growth: 2.0,
            animation: 0.0,
            transition: 0.0,
            spawn_in: 1.2,
            message,
            message_time: 5.0,
            feedback: None,
            pending_sound: None,
            opening_order: MatchBag::default(),
            match_order: MatchBag::default(),
            round_key: None,
            round_spawns: 0,
            math_choices: Vec::new(),
            last_math_choice: None,
            word_choices: Vec::new(),
            last_wrong_word: None,
        }
    }
    pub fn clear_input(&mut self) {
        self.left = false;
        self.right = false;
        self.mouse_x = None;
    }
    fn correct_problem(&mut self, d: Difficulty) -> Problem {
        let target = self.rules.target;
        if self.math_choices.is_empty() {
            self.math_choices = if d.subtraction() {
                (1..=d.max_operand() - target)
                    .map(|right| Problem {
                        left: target + right,
                        right,
                        op: Op::Subtract,
                    })
                    .collect()
            } else {
                // Give each unordered pair equal weight: 5+5 is one family,
                // just like 1+9. Orientation varies separately for practice.
                (1..=target / 2)
                    .map(|left| Problem {
                        left,
                        right: target - left,
                        op: Op::Add,
                    })
                    .collect()
            };
            self.rng.shuffle(&mut self.math_choices);
            let end = self.math_choices.len() - 1;
            if self.math_choices.len() > 1
                && self.last_math_choice.as_ref().is_some_and(|last| {
                    let next = &self.math_choices[end];
                    last.left == next.left && last.right == next.right
                })
            {
                self.math_choices.swap(0, end);
            }
        }
        let mut problem = self.math_choices.pop().unwrap();
        self.last_math_choice = Some(problem);
        if problem.op == Op::Add && problem.left != problem.right && self.rng.fraction() < 0.5 {
            std::mem::swap(&mut problem.left, &mut problem.right);
        }
        problem
    }
    fn wrong_word(&mut self) -> usize {
        if self.word_choices.is_empty() {
            self.word_choices = vocabulary::pool(self.rules.level, self.rules.language)
                .into_iter()
                .filter(|id| *id != self.rules.target_word)
                .collect();
            self.rng.shuffle(&mut self.word_choices);
            let end = self.word_choices.len() - 1;
            if self.word_choices.len() > 1 && self.last_wrong_word == Some(self.word_choices[end]) {
                self.word_choices.swap(0, end);
            }
        }
        let word = self.word_choices.pop().unwrap();
        self.last_wrong_word = Some(word);
        word
    }
    pub fn spawn(&mut self) {
        let target = if self.rules.mode == Mode::Math {
            self.rules.target
        } else {
            self.rules.target_word as i32
        };
        let key = (
            self.rules.mode,
            self.rules.language,
            self.rules.level,
            target,
        );
        if self.round_key != Some(key) {
            self.round_key = Some(key);
            self.round_spawns = 0;
            self.math_choices.clear();
            self.last_math_choice = None;
            self.word_choices.clear();
            self.last_wrong_word = None;
        }
        let matching = if self.round_spawns == 0 {
            let first = self.opening_order.next(&mut self.rng);
            self.match_order = MatchBag::after_first(first, &mut self.rng);
            first
        } else {
            self.match_order.next(&mut self.rng)
        };
        self.round_spawns += 1;
        let d = self.rules.difficulty();
        let content = if self.rules.mode == Mode::Math {
            Content::Math(if matching {
                self.correct_problem(d)
            } else {
                make_problem_with_match(self.rules.target, d, &mut self.rng, false)
            })
        } else {
            Content::Word(if matching {
                self.rules.target_word
            } else {
                self.wrong_word()
            })
        };
        let half_width = (content.label(self.rules.language).chars().count() as f32 * 9.0 + 14.0)
            .clamp(48.0, 300.0);
        let x = half_width + 8.0 + self.rng.fraction() * (PLAY_RIGHT - 2.0 * half_width - 16.0);
        let reading = self.rules.mode == Mode::Reading;
        let speed = if reading {
            d.reading_speed()
        } else {
            d.speed()
        } + self.rng.fraction() * 10.0;
        self.drops.push(Drop {
            x,
            y: if reading { 305.0 } else { 205.0 },
            half_width,
            speed,
            content,
        });
    }
    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        if let Some(feedback) = &mut self.feedback {
            feedback.age += dt;
        }
        if self
            .feedback
            .as_ref()
            .is_some_and(|feedback| feedback.age >= 0.85)
        {
            self.feedback = None;
        }
        self.animation += dt;
        self.message_time = (self.message_time - dt).max(0.0);
        if self.transition > 0.0 {
            self.transition = (self.transition - dt).max(0.0);
            return;
        }
        self.display_growth +=
            (self.rules.growth as f32 - self.display_growth) * (dt * 5.0).min(1.0);
        let direction = self.right as i32 - self.left as i32;
        self.basket_x = self
            .mouse_x
            .unwrap_or(self.basket_x + direction as f32 * 460.0 * dt)
            .clamp(BASKET_HALF + 8.0, PLAY_RIGHT - BASKET_HALF - 8.0);
        self.spawn_in -= dt;
        if self.spawn_in <= 0.0 {
            self.spawn();
            let d = self.rules.difficulty();
            self.spawn_in = if self.rules.mode == Mode::Reading {
                d.reading_interval()
            } else {
                d.interval()
            };
        }
        let mut caught = None;
        for (index, drop) in self.drops.iter_mut().enumerate() {
            let old = drop.y;
            drop.y += drop.speed * dt;
            if old + 21.0 <= BASKET_Y
                && drop.y + 21.0 >= BASKET_Y
                && (drop.x - self.basket_x).abs() <= BASKET_HALF
            {
                caught = Some(index);
                break;
            }
        }
        if let Some(index) = caught {
            let result = match self.drops[index].content {
                Content::Math(problem) => self.rules.catch(problem, &mut self.rng),
                Content::Word(word) => self.rules.catch_word(word, &mut self.rng),
            };
            let kind = if matches!(result, Outcome::Correct | Outcome::LevelUp) {
                FeedbackKind::Correct
            } else {
                FeedbackKind::Wrong
            };
            self.feedback = Some(Feedback {
                kind,
                x: self.basket_x,
                age: 0.0,
            });
            self.pending_sound = Some(kind);
            let language = self.rules.language;
            match result {
                Outcome::LevelUp => {
                    self.message = format!(
                        "{} {}!",
                        language.text(
                            "Fullvuxet! Välkommen till nivå",
                            "Fully grown! Welcome to level"
                        ),
                        self.rules.level
                    );
                    self.display_growth = FULL as f32;
                    self.transition = 1.6;
                }
                Outcome::LevelDown => {
                    self.message = format!(
                        "{} {}. {}",
                        language.text("Tillbaka till nivå", "Back to level"),
                        self.rules.level,
                        language.text("Ditt träd kan växa igen!", "Your tree can regrow!")
                    );
                    self.display_growth = 0.0;
                    self.transition = 1.3;
                }
                Outcome::Correct => {
                    self.message = format!(
                        "{} — {}",
                        self.rules.last_answer,
                        language.text("ditt träd växer!", "your tree grows!")
                    )
                }
                Outcome::Wrong => {
                    self.message = format!(
                        "{} — {}",
                        self.rules.last_answer,
                        language.text("trädet krymper. Försök igen!", "tree shrinks. Try again!")
                    )
                }
            }
            self.round_spawns = 0;
            self.message_time = 3.5;
            self.drops.clear();
            self.spawn_in = 1.0;
        }
        self.drops.retain(|drop| drop.y <= HEIGHT + 30.0);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn state(level: u32, growth: i32, target: i32) -> Rules {
        let mut s = Rules::new(&mut Rng::seeded(7));
        s.level = level;
        s.growth = growth;
        s.target = target;
        s
    }
    #[test]
    fn correct_addition_grows() {
        let mut s = state(1, 2, 5);
        assert_eq!(
            s.catch(
                Problem {
                    left: 2,
                    right: 3,
                    op: Op::Add
                },
                &mut Rng::seeded(7)
            ),
            Outcome::Correct
        );
        assert_eq!((s.growth, s.score), (3, 50));
        assert_ne!(s.target, 5);
    }
    #[test]
    fn correct_subtraction_grows() {
        let mut s = state(3, 2, 2);
        assert_eq!(
            s.catch(
                Problem {
                    left: 3,
                    right: 1,
                    op: Op::Subtract
                },
                &mut Rng::seeded(7)
            ),
            Outcome::Correct
        );
        assert_eq!(s.score, 20);
    }
    #[test]
    fn full_tree_advances() {
        let mut s = state(2, 5, 5);
        assert_eq!(
            s.catch(
                Problem {
                    left: 2,
                    right: 3,
                    op: Op::Add
                },
                &mut Rng::seeded(7)
            ),
            Outcome::LevelUp
        );
        assert_eq!((s.level, s.growth), (3, 2));
        assert!((1..=2).contains(&s.target));
    }
    #[test]
    fn empty_tree_returns() {
        let mut s = state(4, 1, 8);
        assert_eq!(
            s.catch(
                Problem {
                    left: 1,
                    right: 1,
                    op: Op::Add
                },
                &mut Rng::seeded(7)
            ),
            Outcome::LevelDown
        );
        assert_eq!((s.level, s.growth), (3, 3));
        assert!((1..=2).contains(&s.target));
    }
    #[test]
    fn level_one_keeps_seedling() {
        let mut s = state(1, 1, 5);
        for _ in 0..20 {
            s.catch(
                Problem {
                    left: 1,
                    right: 1,
                    op: Op::Add,
                },
                &mut Rng::seeded(7),
            );
        }
        assert_eq!((s.level, s.growth), (1, 1));
    }
    #[test]
    fn only_every_third_level_subtracts() {
        let mut rng = Rng::seeded(42);
        for level in 1..=30 {
            let d = Difficulty(level);
            for target in [d.min_target(), d.max_target()] {
                let mut correct = 0;
                for _ in 0..200 {
                    let p = make_problem(target, d, &mut rng);
                    assert_eq!(
                        p.op,
                        if level % 3 == 0 {
                            Op::Subtract
                        } else {
                            Op::Add
                        }
                    );
                    assert!(p.left > 0 && p.right > 0 && p.answer() > 0);
                    assert!(
                        p.left <= d.operand_limit(target) && p.right <= d.operand_limit(target)
                    );
                    correct += (p.answer() == target) as usize;
                }
                assert!((50..150).contains(&correct));
            }
        }
    }
    #[test]
    fn subtraction_is_simpler_slower_and_progresses() {
        for level in [3, 6, 9, 12] {
            let d = Difficulty(level);
            let previous = Difficulty(level - 1);
            assert!(d.max_operand() < previous.max_operand());
            assert!(d.speed() < previous.speed());
            assert!(d.interval() > previous.interval());
            assert!(Difficulty(level + 3).max_operand() > d.max_operand());
            assert!(Difficulty(level + 3).speed() > d.speed());
        }
        assert_eq!(Difficulty(2).speed() - Difficulty(1).speed(), 5.0);
    }
    #[test]
    fn pause_freezes() {
        let mut game = Game::new();
        game.paused = true;
        game.update(1.0);
        assert_eq!(game.animation, 0.0);
        assert_eq!(game.spawn_in, 1.2);
    }
    #[test]
    fn reading_catch_uses_word_identity_and_advances() {
        for language in [Language::Swedish, Language::English] {
            let mut rng = Rng::seeded(8);
            let mut rules = Rules::for_mode(Mode::Reading, language, &mut rng);
            let word = rules.target_word;
            assert_eq!(WORDS[word].text(language).chars().count(), 3);
            assert_eq!(rules.catch_word(word, &mut rng), Outcome::Correct);
            assert_eq!(rules.growth, 3);
            assert_ne!(rules.target_word, word);
            rules.growth = 5;
            rules.level = 2;
            let word = rules.target_word;
            assert_eq!(rules.catch_word(word, &mut rng), Outcome::LevelUp);
            assert_eq!(WORDS[rules.target_word].text(language).chars().count(), 4);
        }
    }
    #[test]
    fn reading_distractors_are_same_length_and_do_not_match() {
        let mut game = Game::for_mode(Mode::Reading, Language::Swedish);
        let target = game.rules.target_word;
        let mut matches = 0;
        for _ in 0..500 {
            game.spawn();
            let drop = game.drops.last().unwrap();
            if let Content::Word(index) = drop.content {
                assert_eq!(WORDS[index].swedish.chars().count(), 3);
                matches += (index == target) as usize;
            } else {
                panic!("Math problem in reading mode");
            }
        }
        assert!((180..320).contains(&matches));
        let wrong = vocabulary::pool(1, Language::Swedish)
            .into_iter()
            .find(|id| *id != target)
            .unwrap();
        assert_eq!(game.rules.catch_word(wrong, &mut game.rng), Outcome::Wrong);
        assert_eq!(game.rules.target_word, target);
    }
    #[test]
    fn collision_uses_reading_word() {
        let mut game = Game::for_mode(Mode::Reading, Language::Swedish);
        game.drops.push(Drop {
            x: game.basket_x,
            y: BASKET_Y - 22.0,
            half_width: 48.0,
            speed: 100.0,
            content: Content::Word(game.rules.target_word),
        });
        game.update(0.02);
        assert_eq!(game.rules.growth, 3);
        assert_eq!(game.rules.score, 30);
    }
    #[test]
    fn catches_create_one_sound_event_and_finite_visual_feedback() {
        for correct in [true, false] {
            let mut game = Game::for_mode(Mode::Reading, Language::Swedish);
            let word = if correct {
                game.rules.target_word
            } else {
                vocabulary::pool(1, Language::Swedish)
                    .into_iter()
                    .find(|id| *id != game.rules.target_word)
                    .unwrap()
            };
            game.drops.push(Drop {
                x: game.basket_x,
                y: BASKET_Y - 22.0,
                half_width: 48.0,
                speed: 100.0,
                content: Content::Word(word),
            });
            game.update(0.02);
            let expected = if correct {
                FeedbackKind::Correct
            } else {
                FeedbackKind::Wrong
            };
            assert_eq!(game.pending_sound.take(), Some(expected));
            assert_eq!(game.feedback.as_ref().unwrap().kind, expected);
            game.update(0.1);
            assert_eq!(game.pending_sound, None);
            assert!(game.feedback.as_ref().unwrap().age > 0.0);
            game.update(0.8);
            assert!(game.feedback.is_none());
        }
    }
    #[test]
    fn opening_answers_are_balanced_without_long_streaks() {
        for language in [Language::Swedish, Language::English] {
            let mut game = Game::for_mode(Mode::Reading, language);
            game.rng = Rng::seeded(83);
            let mut streak = 0;
            let mut previous = None;
            let mut matches = 0;
            for round in 0..400 {
                // Includes retries on the same target, not just new pictures.
                game.round_spawns = 0;
                game.spawn();
                let correct = matches!(game.drops.last().unwrap().content,Content::Word(id) if id==game.rules.target_word);
                streak = if previous == Some(correct) {
                    streak + 1
                } else {
                    1
                };
                previous = Some(correct);
                assert!(streak <= 2);
                matches += correct as usize;
                if round % 4 == 3 {
                    assert_eq!(matches, (round + 1) / 2);
                }
                if round % 3 == 0 {
                    game.rules.new_target(&mut game.rng);
                }
                game.drops.clear();
            }
        }
    }
    #[test]
    fn each_round_has_even_mix_with_no_long_runs() {
        let mut game = Game::for_mode(Mode::Reading, Language::Swedish);
        game.rng = Rng::seeded(42);
        for _ in 0..30 {
            game.rules.new_target(&mut game.rng);
            let mut correct_count = 0;
            let mut streak = 0;
            let mut last = None;
            for index in 0..40 {
                game.spawn();
                let correct = matches!(game.drops.last().unwrap().content,Content::Word(id) if id==game.rules.target_word);
                correct_count += correct as usize;
                streak = if last == Some(correct) { streak + 1 } else { 1 };
                last = Some(correct);
                assert!(streak <= 2);
                if index % 4 == 3 {
                    assert_eq!(correct_count, (index + 1) / 2);
                }
            }
            game.drops.clear();
        }
    }
    #[test]
    fn goal_ten_cycles_all_addition_families_evenly() {
        let mut game = Game::new();
        game.rng = Rng::seeded(42);
        game.rules.level = 4;
        game.rules.target = 10;
        let mut families = std::collections::BTreeMap::new();
        let mut previous = None;
        let mut index = 0;
        while index < 50 {
            game.spawn();
            let Content::Math(problem) = game.drops.pop().unwrap().content else {
                panic!("Word in math mode");
            };
            if problem.answer() != 10 {
                continue;
            }
            assert_eq!(problem.answer(), 10);
            let family = (
                problem.left.min(problem.right),
                problem.left.max(problem.right),
            );
            assert_ne!(previous, Some(family));
            previous = Some(family);
            *families.entry(family).or_insert(0) += 1;
            if index == 4 {
                assert_eq!(families.len(), 5);
            }
            index += 1;
        }
        assert_eq!(families.len(), 5);
        assert!(families.values().all(|count| *count == 10));
    }
    #[test]
    fn subtraction_variations_cycle_and_single_option_is_safe() {
        let mut game = Game::new();
        game.rules.level = 9;
        game.rules.target = 1;
        let mut choices = std::collections::BTreeSet::new();
        for _ in 0..4 {
            let p = game.correct_problem(Difficulty(9));
            assert_eq!(p.answer(), 1);
            choices.insert((p.left, p.right));
        }
        assert_eq!(choices.len(), 4);
        game.math_choices.clear();
        game.last_math_choice = None;
        game.rules.level = 3;
        game.rules.target = 2;
        for _ in 0..10 {
            let p = game.correct_problem(Difficulty(3));
            assert_eq!((p.left, p.right), (3, 1));
        }
    }
}
