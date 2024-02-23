pub fn word(prefix: Option<&str>) -> String {
    let name_main_part = vec![
        "able", "ache", "arch", "army", "aura", "avid", "axle", "baby", "bane", "bark",
        "beam", "bell", "bird", "bolt", "bump", "cake", "cave", "clip", "cold", "cove",
        "dart", "dash", "dive", "door", "dusk", "earl", "echo", "edge", "emit", "fall",
        "farm", "fizz", "flip", "fuzz", "gift", "girl", "glee", "golf", "gulp", "gush",
        "haze", "hike", "hope", "hush", "icon", "idea", "inky", "iron", "jade", "jinx",
        "jolt", "jump", "keen", "kiss", "kite", "lamb", "lark", "lime", "lush", "math",
        "mint", "moon", "mute", "myth", "nail", "nook", "note", "nova", "numb", "oath",
        "onyx", "ooze", "open", "park", "path", "pave", "puff", "quiz", "rain", "rave",
        "raze", "rush", "seed", "sift", "silk", "tilt", "time", "tint", "toss", "undo",
        "unit", "vast", "vibe", "vice", "wave", "wisp", "yawn", "yell", "yoga", "zero",
        "zest", "zone"
    ];
    let idx = rand(name_main_part.len());
    let selected = name_main_part[idx];
    match prefix {
        Some(p) => format!("{}-{}", p, selected),
        None => selected.to_owned()
    }
}

pub fn random_id() -> String {
    let dict = &[
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 'x', 't',
        'u', 'v', 'w', 'x', 'y', 'z'
    ];

    let digi_1 = rand(36);
    let digi_2 = rand(36);
    let digi_3 = rand(36);
    let digi_4 = rand(36);
    let mut result = String::new();
    result.push(dict[digi_1]);
    result.push(dict[digi_2]);
    result.push(dict[digi_3]);
    result.push(dict[digi_4]);
    result
}

fn rand(cap:usize) -> usize {
    let rand_num: usize = rand::random();
    rand_num % cap
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_random_id() {
        let str_random_id = random_id();
        assert_eq!(str_random_id.len(), 4);

        assert_ne!(random_id(), random_id());
    }

    #[test]
    fn test_main_part() {
        let str_main_part = word(None);
        assert_eq!(str_main_part.len(), 4);
        assert_ne!(word(None), word(None));
    }

    #[test]
    fn test_main_part_with_prefix() {
        let str_main_part = word(Some("test"));
        assert_eq!(str_main_part.len(), 9);
    }
}
