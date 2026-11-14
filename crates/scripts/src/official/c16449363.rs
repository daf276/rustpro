// Scrap Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 16449363;

pub const LISTED_SERIES: &[u16] = &[SET_SYNCHRON, SET_WARRIOR];
pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_WARRIOR];
// A copy in the hand as Synchro material and one in the Graveyard, Junk Warrior in the Extra Deck (Kojikocy, a
// Level 4 Normal Monster, as the other material) and on the field, and Dark Hole in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_JUNK_WARRIOR, EXTRA),
    (CARD_JUNK_WARRIOR, EXTRA),
    (CARD_JUNK_WARRIOR, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(HAND), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, 20932152);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_SYNCHRO_MAT_FROM_HAND);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_value(
        duel,
        Value::MaterialFor(|duel, _, _, sc| sc.lists_archetype_as_material(duel, &[SET_SYNCHRON])),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, |duel, a| {
        duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT | REASON_REPLACE)?;
        Ok(())
    });
    c.register_effect(duel, e2);
}

pub fn junk_warrior_support(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_JUNK_WARRIOR) || (c.is_synchro_monster(duel) && c.is_original_set_card(duel, &[SET_WARRIOR]))
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    junk_warrior_support(c, duel)
        && c.is_faceup(duel)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_status(duel, STATUS_DESTROY_CONFIRMED)
            && c.is_able_to_remove(duel)
            && a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}
