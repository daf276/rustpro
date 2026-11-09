// Primite Drillbeam
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 29095457;

pub const LISTED_NAMES: &[u32] = &[ID];

// Mystical Elf (Normal Monster) in the hand to reveal, Primite Dragon Ether Beryl and a second copy in the Graveyard to Set,
// and an opponent's card to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (15025844, HAND),
    (15025844, MZONE),
    (63198739, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_PRIMITE]));
        duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_PRIMITE]) || (c.is_monster(duel) && c.is_type(duel, TYPE_NORMAL)))
        && !(c.is_public(duel) || c.is_code(duel, ID))
}

fn nocostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_type(duel, TYPE_NORMAL) || (c.is_set_card(duel, &[SET_PRIMITE]) && c.is_level_above(duel, 5)))
        && c.is_faceup(duel)
        && !c.is_type(duel, TYPE_TOKEN)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let b1 = duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, Some(c.into()));
    let b2 = duel.is_existing_matching_card(nocostfilter, tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = if b1 && b2 {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
    } else {
        Some(if b1 { 1 } else { 2 })
    };
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
    }
    Ok(false)
}

fn negatable(c: CardId, duel: &Duel) -> bool {
    c.is_negatable(duel) && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && negatable(chkc, duel) && chkc != c);
    }
    let exception = Some(c.into());
    if chk == 0 {
        return Ok(duel.is_existing_target(negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, exception)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_negatable(duel)
        && tc.is_can_be_disabled_by_effect(duel, e, true)
    {
        tc.negate_effects(duel, e.get_handler(duel), 0, false);
        duel.adjust_instantly(Some(tc));
        if tc.is_disabled(duel) {
            duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
