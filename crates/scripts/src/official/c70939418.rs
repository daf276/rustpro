// Speedroid Gum Prize
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 70939418;

// Takuhee (Level 4 WIND) on the field as a Synchro Material for Hi-Speedroid Glider #2, Speedroid Terrortop in the Deck to mill, Elfnote Lucina
// (13597785) to attack and damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (3170832, MZONE),
    (3170832, MZONE),
    (10732060, EXTRA),
    (10732060, EXTRA),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (81275020, DECK),
    (81275020, DECK),
    (81275020, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DAMAGE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(
        duel,
        TIMING_BATTLE_STEP_END | TIMING_BATTLE_END,
        TIMING_BATTLE_STEP_END | TIMING_BATTLE_END,
    );
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e2.set_target(duel, sctg);
    e2.set_operation(duel, scop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_condition(duel, ddcon);
    e3.set_target(duel, ddtg);
    e3.set_operation(duel, ddop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_battle_phase() && a.ep == a.tp && a.r & (REASON_BATTLE + REASON_EFFECT) != 0
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let reset = RESET_EVENT | (RESETS_STANDARD & !(RESET_TEMP_REMOVE | RESET_TURN_SET)) | RESET_PHASE | PHASE_BATTLE;
        c.register_flag_effect(duel, ID, reset, 0, 1);
    }
    Ok(())
}

fn mfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_WIND)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        let mg = duel.get_matching_group(mfilter, tp, LOCATION_MZONE, 0, None);
        return Ok(!c.is_status(duel, STATUS_CHAINING)
            && duel.is_existing_matching_card(
                |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), Some(mg)),
                tp,
                LOCATION_EXTRA,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.get_controler(duel) != tp || !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let mg = duel.get_matching_group(mfilter, tp, LOCATION_MZONE, 0, None);
    let g = duel.get_matching_group(
        |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), Some(mg)),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.synchro_summon(tp, sc, Some(c.into()), Some(mg))?;
        }
    }
    Ok(())
}

fn ddcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
}

fn ddtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 1);
    Ok(false)
}

fn ddop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.discard_deck(tp, 1, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let tc = duel.get_operated_group().get_first();
    let sync = c.get_reason_card(duel);
    if let Some(tc) = tc
        && tc.is_set_card(duel, &[SET_SPEEDROID])
        && tc.is_monster(duel)
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        // The Lua registers on a nil Synchro Monster when the reason card is gone.
        unported!(duel, sync.is_none());
        if let Some(sync) = sync {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, 1000);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sync.register_effect(duel, e1);
        }
    }
    Ok(())
}
