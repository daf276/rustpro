// Vanquish Soul Jiaolong
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 9091064;
pub const LISTED_NAMES: &[u32] = &[ID];

// Hollie Sue and Dr. Mad Love in the hand to reveal as costs, and a Hollie Sue on the field with Dr. Mad Love to chain from.
pub const SUPPORT: &[(u32, u16)] = &[
    (93156774, HAND),
    (93156774, HAND),
    (29280200, HAND),
    (29280200, HAND),
    (ID, HAND),
    (ID, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CONFIRM);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e2.set_target(duel, vstg);
    e2.set_operation(duel, vsop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_activated(duel) && re.get_handler(duel).is_set_card(duel, &[SET_VANQUISH_SOUL]))
        && a.r & REASON_COST > 0
        && a.rp == a.tp
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_HAND), 1, None))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn vscostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && !c.is_public(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn vstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cg = duel.get_matching_group(vscostfilter, tp, LOCATION_HAND, 0, None);
    let b1 = !cg.is_empty() && duel.is_existing_matching_card(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    let b2 = cg.len() > 1 && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel
        .select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
        .unwrap_or(0);
    e.set_label(duel, op as i64);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = cg.select(duel, tp, op as u8, op as u8, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    if op == 1 {
        e.set_category(duel, CATEGORY_POSITION);
    } else if op == 2 {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn vsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let g = duel.select_matching_card(tp, CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.change_position_full(
                g,
                [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
                false,
            )?;
        }
    } else if op == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}
