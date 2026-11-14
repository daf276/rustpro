// Vanquish Soul Dr. Mad Love
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 29280200;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// Dr. Mad Love with Jiaolong (FIRE), Hollie Sue (EARTH) and another copy (DARK) in the hand to reveal, and monsters on both sides.
// No Vanquish Soul Spell/Trap is ported, so the search is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (93156774, HAND),
    (93156774, HAND),
    (9091064, DECK),
    (3573512, MZONE | OPPONENT),
    (4035199, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E | TIMING_DAMAGE_STEP,
    );
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e3.set_target(duel, vstg);
    e3.set_operation(duel, vsop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn vscostfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_attribute(duel, att) && !c.is_public(duel)
}

fn vsrescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    let count = |att| sg.filter_count(duel, |c, duel| c.is_attribute(duel, att), None);
    (count(ATTRIBUTE_DARK) > 0 && count(ATTRIBUTE_EARTH) > 0, false)
}

fn vstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let not_dmg_step = duel.get_current_phase() != PHASE_DAMAGE;
    let cg1 = duel.get_matching_group(|c, duel| vscostfilter(c, duel, ATTRIBUTE_DARK), tp, LOCATION_HAND, 0, None);
    let fg1 = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    let b1 = !cg1.is_empty() && !fg1.is_empty() && (not_dmg_step || !duel.is_damage_calculated());
    let earth = duel.get_matching_group(|c, duel| vscostfilter(c, duel, ATTRIBUTE_EARTH), tp, LOCATION_HAND, 0, None);
    let cg2 = cg1.union(earth);
    let fg2 = duel.get_matching_group(
        aux::faceup_filter(CardId::is_able_to_hand),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    let b2 = !fg2.is_empty() && aux::select_unselect_group_check(duel, cg2, 1, 2, &|duel, sg, _, _| vsrescon(duel, sg)) && not_dmg_step;
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = cg1.select(duel, tp, 1, 1, None)?;
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        e.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    } else if op == Some(2) {
        let rescon = |duel: &Duel, sg, _, _| vsrescon(duel, sg);
        let g = aux::select_unselect_group_full(duel, cg2, 1, 2, &rescon, tp, HINTMSG_CONFIRM, Some(&rescon), false)?;
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        e.set_category(duel, CATEGORY_TOHAND);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    }
    Ok(false)
}

fn vsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    let c = e.get_handler(duel);
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
        let Some(tc) = duel
            .select_matching_card(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.hint_selection(tc);
        tc.update_attack(duel, -500, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
        tc.update_defense(duel, -500, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
    } else if op == 2 {
        let g = duel.get_matching_group(
            aux::faceup_filter(|c, duel| c.is_defense_above(duel, 0)),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            None,
        );
        let Some((mut g2, _)) = g.get_min_group(duel, |c, duel| c.get_defense(duel).into()) else {
            return Ok(());
        };
        if g2.len() > 1 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
            g2 = g2.filter_select(duel, tp, CardId::is_able_to_hand, 1, 1, None)?;
        }
        if g2.is_empty() {
            return Ok(());
        }
        duel.hint_selection(g2);
        duel.send_to_hand(g2, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
