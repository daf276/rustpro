// D/D/D Wave King Caesar
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 3758046;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_FIEND, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// On the field with Xyz Import to attach Sangan, Dark Hole to destroy monsters and Dark Contract with the Witch in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (67322708, MZONE),
    (53129443, HAND),
    (9765723, DECK),
    (9765723, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_able_to_enter_bp() || duel.is_battle_phase());
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, regtg);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn regtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_ALL, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, a.tp, 1000);
    Ok(false)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, turn: i32) -> bool {
    c.is_reason(duel, REASON_DESTROY) && c.get_turn_id(duel) == turn && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (e, tp, turn) = (a.e, a.tp, duel.get_turn_count());
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, turn)),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, turn) = (a.e, a.tp, duel.get_turn_count());
    duel.hint(HINT_CARD, 0, ID as u64);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft == 0 {
        return Ok(());
    }
    let mut dg = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, turn)),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if dg.is_empty() {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    if dg.len() > ft as usize {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        dg = dg.select(duel, tp, ft as u8, ft as u8, None)?;
    }
    if !dg.is_empty() && duel.special_summon(dg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let ct = duel.get_operated_group().len();
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_count_limit(duel, 1, 0);
        e1.set_label(duel, ct as i64);
        e1.set_operation(duel, damop);
        e1.set_reset(duel, RESET_PHASE | PHASE_STANDBY);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ct = a.e.get_label(duel);
    duel.damage(a.tp, ct as u32 * 1000, REASON_EFFECT)?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_CONTRACT]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
