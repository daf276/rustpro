// Branded Regained
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 34090915;

pub const LISTED_SERIES: &[u16] = &[SET_BYSTIAL];
// On the field with Bystial Magnamhut in hand to banish the DARK Bystials in the Graveyard, which are also the ones to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (33854624, HAND),
    (33854624, HAND),
    (32731036, GRAVE),
    (32731036, GRAVE),
    (27572350, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CUSTOM + ID);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| !duel.is_phase(PHASE_DAMAGE));
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    e2.set_label_object_group(duel, Some(CardSet::default()));
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_REMOVE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_label_object(duel, Some(e2));
    e3.set_operation(duel, regop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e4.set_condition(duel, spcond);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e5);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    let dark_light = ATTRIBUTE_DARK | ATTRIBUTE_LIGHT;
    c.is_attribute(duel, dark_light)
        && c.is_faceup(duel)
        && (!c.is_previous_location(duel, LOCATION_ONFIELD)
            || (c.get_previous_type_on_field(duel) & TYPE_MONSTER > 0 && c.get_previous_attribute_on_field(duel) & dark_light > 0))
        && c.is_able_to_deck(duel)
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_REMOVED)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_phase(PHASE_DAMAGE) {
        return Ok(());
    }
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| cfilter(c, duel, e), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e2) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e2.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g = g.union(tg).filter(duel, |c, duel| c.get_flag_effect(duel, ID) != 0, None);
    e2.set_label_object_group(duel, Some(g));
    if duel.get_flag_effect(tp, ID) == 0 {
        duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
        duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    }
    Ok(())
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| cfilter(c, duel, e), None);
    if let Some(chkc) = chkc {
        return Ok(g.contains(chkc) && cfilter(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(!g.is_empty() && duel.is_player_can_draw(tp, 1));
    }
    duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tg = g.select(duel, tp, 1, 1, None)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
    {
        duel.draw(a.tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcond(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BYSTIAL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
            && duel.get_flag_effect(tp, ID + 1) == 0);
    }
    duel.register_flag_effect(tp, ID + 1, RESET_CHAIN, 0, 1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}
