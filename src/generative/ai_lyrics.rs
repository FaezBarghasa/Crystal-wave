#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RhymeScheme {
    #[default]
    AABB,
    ABAB,
    ABBA,
    FreeVerse,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LyricLine {
    pub line_index: usize,
    pub text: String,
    pub syllable_count: usize,
    pub rhyme_label: char,
}

pub struct AiLyrics {
    pub rhyme_scheme: RhymeScheme,
}

impl Default for AiLyrics {
    fn default() -> Self {
        Self { rhyme_scheme: RhymeScheme::ABAB }
    }
}

impl AiLyrics {
    pub fn new(rhyme_scheme: RhymeScheme) -> Self {
        Self { rhyme_scheme }
    }

    pub fn format_stanza(&self, lines: &[&str]) -> Vec<LyricLine> {
        let labels = match self.rhyme_scheme {
            RhymeScheme::AABB => ['A', 'A', 'B', 'B'],
            RhymeScheme::ABAB => ['A', 'B', 'A', 'B'],
            RhymeScheme::ABBA => ['A', 'B', 'B', 'A'],
            RhymeScheme::FreeVerse => ['A', 'B', 'C', 'D'],
        };

        lines
            .iter()
            .enumerate()
            .map(|(i, &txt)| {
                let syllables = txt.split_whitespace().count() * 2; // Approximation
                LyricLine {
                    line_index: i,
                    text: txt.to_string(),
                    syllable_count: syllables,
                    rhyme_label: labels[i % 4],
                }
            })
            .collect()
    }
}
