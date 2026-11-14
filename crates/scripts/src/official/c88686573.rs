// Nimble Angler
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 88686573;

pub const LISTED_SERIES: &[u16] = &[SET_NIMBLE];
// Copies in the Deck sent to the GY by Foolish Burial, with Nimble Beaver and Momonga in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (81439173, HAND),
    (81439173, HAND),
    (68353324, DECK),
    (68353324, DECK),
    (22567609, DECK),
    (22567609, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(68353324)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND | LOCATION_DECK)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NIMBLE])
        && c.get_code(duel) != ID
        && c.is_level_below(duel, 3)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    ft = ft.min(2);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, ft as u8, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
