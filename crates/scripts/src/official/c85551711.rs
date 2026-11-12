// Ebon High Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 85551711;

// In the Extra Deck, Xyz Summoned with Level 7 Dark Magicians, with the opponent's Raigeki to destroy it, and Dark Magicians to
// Special Summon; Mystical Space Typhoon and Trap Hole in hand for the activation effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (36996508, MZONE),
    (46986414, MZONE),
    (46986415, MZONE),
    (46986416, DECK),
    (46986417, DECK),
    (36996508, DECK),
    (12580477, HAND | OPPONENT),
    (5318639, HAND),
    (4206964, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, 0);
    e1.set_condition(duel, handcon);
    e1.set_value(
        duel,
        Value::Activation(|duel, e, re| {
            let fieldid = e.get_handler(duel).get_field_id(duel);
            re.get_handler(duel)
                .register_flag_effect_full(duel, ID, RESET_CHAIN, 0, 0, i64::from(fieldid), 0);
        }),
    );
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_ACTIVATE_COST);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_target_activate_cost(duel, costtg);
    e3.set_operation(duel, costop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_condition(duel, spcon);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn handcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.is_turn_player(1 - tp) && a.e.get_handler(duel).check_remove_overlay_card(duel, tp, 1, REASON_EFFECT)
}

fn costtg(duel: &Duel, e: EffectId, te: EffectId, tp: u8) -> bool {
    let tc = te.get_handler(duel);
    duel.is_turn_player(1 - tp)
        && tc.is_location(duel, LOCATION_HAND)
        && tc.get_flag_effect(duel, ID) > 0
        && tc.get_flag_effect_label(duel, ID) == Some(i64::from(e.get_handler(duel).get_field_id(duel)))
}

fn costop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID as u64);
    a.e.get_handler(duel).remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)?;
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if !(c.is_previous_location(duel, LOCATION_MZONE) && c.is_xyz_summoned(duel)) {
        return false;
    }
    c.is_reason(duel, REASON_BATTLE) || (a.rp == 1 - a.tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, a.tp))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        if dg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let sg = dg.select(duel, tp, 1, 1, None)?;
        duel.hint_selection_full(sg, true);
        duel.break_effect()?;
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}
