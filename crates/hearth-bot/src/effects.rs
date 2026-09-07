//! Recognized card descriptions for heuristic evaluation.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Effect {
    Damage(i32),
    Heal(i32),
    Draw(i32),
    Buff(i32, i32, bool),
}

pub(super) fn effect(text: &str) -> Option<Effect> {
    let mut clean = String::new();
    let mut tag = false;
    for c in text.chars() {
        match c {
            '<' => tag = true,
            '>' => tag = false,
            '$' | '#' => {}
            _ if !tag => clean.push(c),
            _ => {}
        }
    }
    let text = clean
        .replace("[x]", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for (prefix, suffix, kind) in [
        ("Deal ", " damage.", 0),
        ("Restore ", " Health.", 1),
        ("Draw ", " cards.", 2),
    ] {
        if let Some(amount) = text
            .strip_prefix(prefix)
            .and_then(|s| s.strip_suffix(suffix))
            .and_then(|s| s.parse::<i32>().ok())
        {
            return Some(match kind {
                0 => Effect::Damage(amount),
                1 => Effect::Heal(amount),
                _ => Effect::Draw(amount),
            });
        }
    }
    if text == "Draw a card." {
        return Some(Effect::Draw(1));
    }
    // Audited compound buffs from the former training curriculum.
    match text.as_str() {
        "Give a minion +4/+4. (+4 Attack/+4 Health)" => return Some(Effect::Buff(4, 4, true)),
        "Give a minion +2/+4 and Spell Damage +1." => return Some(Effect::Buff(2, 4, true)),
        "Give a minion +2/+6 and Taunt. When it dies, summon a Stegodon." => {
            return Some(Effect::Buff(2, 6, true));
        }
        "Give a minion +1/+1 and \"Deathrattle: Get an Explorer's Hat.\"" => {
            return Some(Effect::Buff(1, 1, true));
        }
        _ => {}
    }
    let permanent = !text.ends_with(" this turn.");
    let stats = text
        .strip_prefix("Give a minion +")
        .and_then(|s| s.strip_suffix(if permanent { "." } else { " this turn." }));
    if let Some((attack, health)) = stats.and_then(|s| s.split_once("/+")) {
        return Some(Effect::Buff(
            attack.parse().ok()?,
            health.parse().ok()?,
            permanent,
        ));
    }
    if let Some(attack) = text
        .strip_prefix("Give a minion +")
        .and_then(|s| s.strip_suffix(" Attack."))
    {
        return Some(Effect::Buff(attack.parse().ok()?, 0, true));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrated_buff_descriptions_cover_all_seven_original_cards() {
        for (text, attack, health) in [
            ("Give a minion +3\u{a0}Attack.", 3, 0),
            ("Give a minion +4/+4. <i>(+4 Attack/+4 Health)</i>", 4, 4),
            ("Give a minion +2/+6.", 2, 6),
            ("Give a minion +1/+2.", 1, 2),
            ("Give a minion +2/+4 and <b>Spell Damage +1</b>.", 2, 4),
            (
                "Give a minion +1/+1 and \"<b>Deathrattle:</b> Get an Explorer's Hat.\"",
                1,
                1,
            ),
            (
                "Give a minion +2/+6 and <b>Taunt</b>. When it dies, summon a Stegodon.",
                2,
                6,
            ),
        ] {
            assert_eq!(
                effect(text),
                Some(Effect::Buff(attack, health, true)),
                "{text}"
            );
        }
        assert_eq!(
            effect("Give a minion +4/+4. Destroy it at the end of the turn."),
            None
        );
    }
}
