// Chaos-End Master
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 46609443;

// Copies on the field with weak monsters (Key Mace) on the opponent's side to destroy in battle, and Level 5 Normal Monsters
// (Shapesnatch, Leo Wizard) in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (4035199, DECK),
    (4392470, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(eg) = a.eg else { return false };
    eg.get_first().is_some_and(|tc| {
        eg.len() == 1
            && tc.get_reason_card(duel) == Some(a.e.get_handler(duel))
            && tc.is_location(duel, LOCATION_GRAVE)
            && tc.is_reason(duel, REASON_BATTLE)
    })
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attack_below(duel, 1600) && c.is_level_above(duel, 5) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, e, tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| filter(c, duel, e, tp), LOCATION_DECK, POS_FACEUP)
}
