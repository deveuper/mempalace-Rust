#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

/// Emotion codes for AAAK dialect
/// Universal emotion abbreviations
pub const EMOTION_CODES: &[(&str, &str)] = &[
    // Core emotions
    ("vulnerability", "vul"),
    ("vulnerable", "vul"),
    ("joy", "joy"),
    ("joyful", "joy"),
    ("fear", "fear"),
    ("mild_fear", "fear"),
    ("trust", "trust"),
    ("trust_building", "trust"),
    ("grief", "grief"),
    ("raw_grief", "grief"),
    ("wonder", "wonder"),
    ("philosophical_wonder", "wonder"),
    ("rage", "rage"),
    ("anger", "rage"),
    ("love", "love"),
    ("hope", "hope"),
    ("despair", "despair"),
    ("peace", "peace"),
    ("humor", "humor"),
    ("tenderness", "tender"),
    ("raw_honesty", "raw"),
    ("self_doubt", "doubt"),
    ("relief", "relief"),
    ("anxiety", "anx"),
    ("exhaustion", "exhaust"),
    ("conviction", "convict"),
    ("quiet_passion", "passion"),
    // Extended emotions
    ("excitement", "excit"),
    ("curiosity", "curious"),
    ("frustration", "frust"),
    ("satisfaction", "satis"),
    ("disappointment", "disap"),
    ("confidence", "conf"),
    ("confusion", "confus"),
    ("clarity", "clar"),
    ("surprise", "surp"),
    ("anticipation", "antic"),
    ("nostalgia", "nost"),
    ("gratitude", "grat"),
    ("pride", "pride"),
    ("shame", "shame"),
    ("guilt", "guilt"),
    ("envy", "envy"),
    ("compassion", "comp"),
    ("empathy", "emp"),
    ("loneliness", "lone"),
    ("belonging", "belong"),
];

/// Get emotion code for a given emotion name
pub fn get_emotion_code(emotion: &str) -> Option<&'static str> {
    let emotion_lower = emotion.to_lowercase();
    EMOTION_CODES
        .iter()
        .find(|(name, _)| name.to_lowercase() == emotion_lower)
        .map(|(_, code)| *code)
}

/// Get emotion name for a given code
pub fn get_emotion_name(code: &str) -> Option<&'static str> {
    EMOTION_CODES
        .iter()
        .find(|(_, c)| *c == code)
        .map(|(name, _)| *name)
}
