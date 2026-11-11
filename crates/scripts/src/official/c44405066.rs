// Red-Eyes Flare Metal Dragon
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 44405066;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Red-Eyes Black Dragons, with one in the Graveyard to summon and the opponent's Dark
// Hole to take damage from.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_REDEYES_B_DRAGON, MZONE),
    (CARD_REDEYES_B_DRAGON, MZONE),
    (CARD_REDEYES_B_DRAGON, GRAVE),
    (53129443, HAND | OPPONENT),
];
// The 500 damage is a continuous effect with no chain link, so any damage counts.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0), Check::Message(MSG_DAMAGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, damcon);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e4);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET) | RESET_CHAIN, 0, 1);
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_overlay_count(duel) > 0 && a.ep == 1 - a.tp && c.has_flag_effect(duel, ID)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID.into());
    duel.damage(1 - a.tp, 500, REASON_EFFECT)?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RED_EYES]) && c.is_type(duel, TYPE_NORMAL) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
