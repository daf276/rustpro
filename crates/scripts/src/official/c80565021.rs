// Angelechy Opening to e4
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 80565021;

// In the hand with the Field Spell Angelechy Problem in the Deck, and Angelechy Destrier, Level 7, and Angelechy Shatranga in
// the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (17782288, DECK), (55393975, EXTRA), (42410161, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e4a = duel.create_effect(c);
    e4a.set_type(duel, EFFECT_TYPE_SINGLE);
    e4a.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e4a.set_condition(duel, |duel, a| {
        duel.get_turn_count() == 1 && duel.is_standby_phase_full(Some(1 - a.e.get_handler_player(duel)))
    });
    c.register_effect(duel, e4a);
    let e4b = duel.create_effect(c);
    e4b.set_description(duel, aux::stringid(ID, 0));
    e4b.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4b.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e4b.set_code(duel, EVENT_FREE_CHAIN);
    e4b.set_target(duel, pltg);
    e4b.set_operation(duel, plop);
    e4b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e4b);
}

fn plfilter(c: CardId, duel: &Duel, field: bool) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && !c.is_forbidden(duel) && (!field || c.is_field_spell(duel))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[2, 7])
        && c.is_set_card(duel, &[SET_ANGELECHY])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex_full(tp, tp, None, c, ZONES_EMZ) > 0
        && duel.is_existing_matching_card(|pc, duel| plfilter(pc, duel, false), tp, LOCATION_EXTRA, 0, 1, Some(c.into()))
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mut stzone_count = duel.get_location_count(tp, LOCATION_SZONE);
        if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && !e.get_handler(duel).is_location(duel, LOCATION_SZONE) {
            stzone_count -= 1;
        }
        return Ok(stzone_count > 0
            && duel.is_existing_matching_card(
                |c, duel| plfilter(c, duel, true),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            )
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let filter = aux::necro_valley_filter(|c, duel| plfilter(c, duel, true));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.place_field_spell(tc, tp)?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if !g.is_empty()
            && duel.special_summon_full(g, 0, tp, tp, false, false, POS_FACEUP, ZONES_EMZ)? > 0
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
            let g = duel.select_matching_card(tp, |c, duel| plfilter(c, duel, false), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
            if let Some(sc) = g.get_first()
                && duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
            {
                let e4 = duel.create_effect(c);
                e4.set_type(duel, EFFECT_TYPE_SINGLE);
                e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e4.set_code(duel, EFFECT_CHANGE_TYPE);
                e4.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
                e4.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
                sc.register_effect(duel, e4);
            }
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let reset_count = if duel.is_turn_player(tp) { 2 } else { 1 };
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e4.set_target_range(duel, 1, 0);
    e4.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_synchro_monster(duel)
    });
    e4.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, reset_count);
    duel.register_effect(e4, tp);
    Ok(())
}
