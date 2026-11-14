use crate::language::Language;
pub struct Word {
    pub swedish: &'static str,
    pub english: &'static str,
}
impl Word {
    pub fn text(&self, language: Language) -> &'static str {
        language.text(self.swedish, self.english)
    }
}
pub static WORDS: [Word; 120] = [
    Word {
        swedish: "sten",
        english: "rock",
    },
    Word {
        swedish: "sol",
        english: "sun",
    },
    Word {
        swedish: "måne",
        english: "moon",
    },
    Word {
        swedish: "moln",
        english: "cloud",
    },
    Word {
        swedish: "stjärna",
        english: "star",
    },
    Word {
        swedish: "träd",
        english: "tree",
    },
    Word {
        swedish: "blad",
        english: "leaf",
    },
    Word {
        swedish: "blomma",
        english: "flower",
    },
    Word {
        swedish: "ros",
        english: "rose",
    },
    Word {
        swedish: "buske",
        english: "bush",
    },
    Word {
        swedish: "kulle",
        english: "hill",
    },
    Word {
        swedish: "snö",
        english: "snow",
    },
    Word {
        swedish: "eld",
        english: "fire",
    },
    Word {
        swedish: "ekollon",
        english: "acorn",
    },
    Word {
        swedish: "snäcka",
        english: "shell",
    },
    Word {
        swedish: "katt",
        english: "cat",
    },
    Word {
        swedish: "hund",
        english: "dog",
    },
    Word {
        swedish: "tjur",
        english: "bull",
    },
    Word {
        swedish: "får",
        english: "sheep",
    },
    Word {
        swedish: "gris",
        english: "pig",
    },
    Word {
        swedish: "häst",
        english: "horse",
    },
    Word {
        swedish: "mus",
        english: "mouse",
    },
    Word {
        swedish: "råtta",
        english: "rat",
    },
    Word {
        swedish: "hare",
        english: "hare",
    },
    Word {
        swedish: "kanin",
        english: "rabbit",
    },
    Word {
        swedish: "räv",
        english: "fox",
    },
    Word {
        swedish: "varg",
        english: "wolf",
    },
    Word {
        swedish: "björn",
        english: "bear",
    },
    Word {
        swedish: "lejon",
        english: "lion",
    },
    Word {
        swedish: "tiger",
        english: "tiger",
    },
    Word {
        swedish: "zebra",
        english: "zebra",
    },
    Word {
        swedish: "giraff",
        english: "giraffe",
    },
    Word {
        swedish: "apa",
        english: "ape",
    },
    Word {
        swedish: "panda",
        english: "panda",
    },
    Word {
        swedish: "koala",
        english: "koala",
    },
    Word {
        swedish: "älg",
        english: "moose",
    },
    Word {
        swedish: "hjort",
        english: "deer",
    },
    Word {
        swedish: "groda",
        english: "frog",
    },
    Word {
        swedish: "orm",
        english: "snake",
    },
    Word {
        swedish: "ödla",
        english: "lizard",
    },
    Word {
        swedish: "snigel",
        english: "snail",
    },
    Word {
        swedish: "spindel",
        english: "spider",
    },
    Word {
        swedish: "myra",
        english: "ant",
    },
    Word {
        swedish: "fluga",
        english: "fly",
    },
    Word {
        swedish: "geting",
        english: "wasp",
    },
    Word {
        swedish: "anka",
        english: "duck",
    },
    Word {
        swedish: "gås",
        english: "goose",
    },
    Word {
        swedish: "höna",
        english: "hen",
    },
    Word {
        swedish: "tupp",
        english: "rooster",
    },
    Word {
        swedish: "uggla",
        english: "owl",
    },
    Word {
        swedish: "örn",
        english: "eagle",
    },
    Word {
        swedish: "svan",
        english: "swan",
    },
    Word {
        swedish: "pingvin",
        english: "penguin",
    },
    Word {
        swedish: "fisk",
        english: "fish",
    },
    Word {
        swedish: "haj",
        english: "shark",
    },
    Word {
        swedish: "val",
        english: "whale",
    },
    Word {
        swedish: "delfin",
        english: "dolphin",
    },
    Word {
        swedish: "säl",
        english: "seal",
    },
    Word {
        swedish: "krabba",
        english: "crab",
    },
    Word {
        swedish: "räka",
        english: "shrimp",
    },
    Word {
        swedish: "hamster",
        english: "hamster",
    },
    Word {
        swedish: "pelikan",
        english: "pelican",
    },
    Word {
        swedish: "äpple",
        english: "apple",
    },
    Word {
        swedish: "päron",
        english: "pear",
    },
    Word {
        swedish: "banan",
        english: "banana",
    },
    Word {
        swedish: "citron",
        english: "lemon",
    },
    Word {
        swedish: "apelsin",
        english: "orange",
    },
    Word {
        swedish: "melon",
        english: "melon",
    },
    Word {
        swedish: "druva",
        english: "grape",
    },
    Word {
        swedish: "kiwi",
        english: "kiwi",
    },
    Word {
        swedish: "mango",
        english: "mango",
    },
    Word {
        swedish: "persika",
        english: "peach",
    },
    Word {
        swedish: "plommon",
        english: "plum",
    },
    Word {
        swedish: "körsbär",
        english: "cherry",
    },
    Word {
        swedish: "morot",
        english: "carrot",
    },
    Word {
        swedish: "tomat",
        english: "tomato",
    },
    Word {
        swedish: "potatis",
        english: "potato",
    },
    Word {
        swedish: "lök",
        english: "onion",
    },
    Word {
        swedish: "majs",
        english: "corn",
    },
    Word {
        swedish: "bröd",
        english: "bread",
    },
    Word {
        swedish: "ost",
        english: "cheese",
    },
    Word {
        swedish: "kaka",
        english: "cookie",
    },
    Word {
        swedish: "tårta",
        english: "cake",
    },
    Word {
        swedish: "pizza",
        english: "pizza",
    },
    Word {
        swedish: "muffin",
        english: "muffin",
    },
    Word {
        swedish: "ägg",
        english: "egg",
    },
    Word {
        swedish: "korv",
        english: "sausage",
    },
    Word {
        swedish: "hus",
        english: "house",
    },
    Word {
        swedish: "slott",
        english: "castle",
    },
    Word {
        swedish: "torn",
        english: "tower",
    },
    Word {
        swedish: "bro",
        english: "bridge",
    },
    Word {
        swedish: "tält",
        english: "tent",
    },
    Word {
        swedish: "båt",
        english: "boat",
    },
    Word {
        swedish: "bil",
        english: "car",
    },
    Word {
        swedish: "buss",
        english: "bus",
    },
    Word {
        swedish: "tåg",
        english: "train",
    },
    Word {
        swedish: "cykel",
        english: "bike",
    },
    Word {
        swedish: "raket",
        english: "rocket",
    },
    Word {
        swedish: "traktor",
        english: "tractor",
    },
    Word {
        swedish: "stol",
        english: "chair",
    },
    Word {
        swedish: "bord",
        english: "table",
    },
    Word {
        swedish: "säng",
        english: "bed",
    },
    Word {
        swedish: "soffa",
        english: "sofa",
    },
    Word {
        swedish: "fönster",
        english: "window",
    },
    Word {
        swedish: "lampa",
        english: "lamp",
    },
    Word {
        swedish: "klocka",
        english: "clock",
    },
    Word {
        swedish: "spegel",
        english: "mirror",
    },
    Word {
        swedish: "bok",
        english: "book",
    },
    Word {
        swedish: "penna",
        english: "pencil",
    },
    Word {
        swedish: "linjal",
        english: "ruler",
    },
    Word {
        swedish: "nyckel",
        english: "key",
    },
    Word {
        swedish: "tallrik",
        english: "plate",
    },
    Word {
        swedish: "handduk",
        english: "towel",
    },
    Word {
        swedish: "present",
        english: "gift",
    },
    Word {
        swedish: "hammare",
        english: "hammer",
    },
    Word {
        swedish: "tärning",
        english: "dice",
    },
    Word {
        swedish: "batteri",
        english: "battery",
    },
    Word {
        swedish: "telefon",
        english: "phone",
    },
    Word {
        swedish: "diamant",
        english: "diamond",
    },
    Word {
        swedish: "ballong",
        english: "balloon",
    },
];
pub static IMAGES: [&[u8]; 120] = include!(concat!(env!("OUT_DIR"), "/reading_images.rs"));
pub fn word_length(level: u32) -> usize {
    (3 + level.saturating_sub(1) as usize / 2).min(7)
}
pub fn pool(level: u32, language: Language) -> Vec<usize> {
    let length = word_length(level);
    WORDS
        .iter()
        .enumerate()
        .filter(|(_, word)| word.text(language).chars().count() == length)
        .map(|(index, _)| index)
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bilingual_vocabulary_is_graded() {
        assert_eq!(WORDS.len(), 120);
        for language in [Language::Swedish, Language::English] {
            let mut unique = std::collections::HashSet::new();
            for word in &WORDS {
                assert!((3..=7).contains(&word.text(language).chars().count()));
                assert!(unique.insert(word.text(language)));
            }
            for level in 1..=20 {
                let words = pool(level, language);
                assert!(words.len() >= 2);
                assert!(words
                    .iter()
                    .all(|id| WORDS[*id].text(language).chars().count() == word_length(level)));
            }
        }
    }
}
