// Emblema Oath
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 77765207;

// With Centur-Ion monsters and Stand Up Centur-Ion! in the Deck to place or Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (42493140, DECK),
    (96030710, DECK),
    (41371602, DECK),
    (41371602, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_monster(duel) && !c.is_forbidden(duel)
}

fn setfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_spell_trap(duel) && c.is_ssetable(duel) && (ft > 0 || c.is_field_spell(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_SZONE);
    if e.get_handler(duel).is_location(duel, LOCATION_HAND) {
        ft -= 1;
    }
    let b1 = ft > 0 && duel.is_existing_matching_card(plfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_existing_matching_card(|c, duel| setfilter(c, duel, ft), tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_SET);
    }
    Ok(false)
}

fn spconfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_faceup(duel) && (c.get_flag_effect(duel, ID) > 0 || c.is_original_code_rule(duel, code))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    if op == 1 {
        if ft <= 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
        {
            sc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
            let c = e.get_handler(duel);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_CHANGE_TYPE);
            e1.set_value(duel, (TYPE_TRAP | TYPE_CONTINUOUS) as i32);
            e1.set_reset(duel, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET);
            sc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_description(duel, aux::stringid(ID, 3));
            e2.set_type(duel, EFFECT_TYPE_FIELD);
            e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e2.set_target_range(duel, 1, 0);
            e2.set_label(duel, i64::from(sc.get_original_code_rule(duel)));
            e2.set_condition(duel, |duel, a| {
                let code = a.e.get_label(duel) as u32;
                duel.is_existing_matching_card(
                    |c, duel| spconfilter(c, duel, code),
                    a.e.get_handler_player(duel),
                    LOCATION_ONFIELD,
                    0,
                    1,
                    None,
                )
            });
            e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
                !c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_location(duel, LOCATION_EXTRA)
            });
            e2.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e2, tp);
            aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_set_card(duel, &[SET_CENTUR_ION]));
        }
    } else if op == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let g = duel.select_matching_card(tp, |c, duel| setfilter(c, duel, ft), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.sset(tp, g)?;
        }
    }
    Ok(())
}
