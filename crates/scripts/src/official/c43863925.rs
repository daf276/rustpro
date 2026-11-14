// Exosister Stella
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 43863925;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];
pub const LISTED_NAMES: &[u32] = &[16474916];

// On the field with Exosister Elis in the hand to summon and on the field, and the opponent's Monster Reborn to take a monster out of a
// Graveyard, for Exosister Karmael in the Extra Deck to be Xyz Summoned.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (16474916, HAND),
    (16474916, HAND),
    (16474916, MZONE),
    (77675029, EXTRA),
    (77675029, EXTRA),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (15023985, GRAVE),
    (15023985, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_LEAVE_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_target(duel, super::c16474916::sptg);
    e2.set_operation(duel, super::c16474916::spop);
    c.register_effect(duel, e2);
}

fn hspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_EXOSISTER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn hsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| hspfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| hspfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, 16474916)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
    {
        duel.break_effect()?;
        duel.recover(tp, 800, REASON_EFFECT)?;
    }
    Ok(())
}
