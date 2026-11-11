// Astrazma Urania
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 18738846;

// In the hand, with monsters of different Levels in the hand and one on the field to return, an opponent's Sangan and
// Elfnote Lucina to negate, and the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (12482652, HAND),
    (4035199, HAND),
    (26202165, HAND),
    (12482652, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, (ID, 0), EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(CardId::is_able_to_hand_as_cost, tp, LOCATION_MZONE, 0, None);
    !g.is_empty() && duel.get_mzone_count(tp, Some(g)) > 0
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_able_to_hand_as_cost, tp, LOCATION_MZONE, 0, None);
    let sg = aux::select_unselect_group_full(duel, g, 1, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_RTOHAND, None, true)?;
    if sg.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(sg));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_COST)?;
    }
    Ok(())
}

fn discostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.has_level(duel) && !c.is_public(duel)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(discostfilter, tp, LOCATION_HAND, 0, None);
    let max_target_count = duel.get_target_count(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty() && max_target_count > 0);
    }
    let max_reveal_count = max_target_count.min(g.get_class_count(duel, |c, duel| i64::from(c.get_level(duel))));
    let sg = aux::select_unselect_group(
        duel,
        g,
        1,
        max_reveal_count,
        &aux::dpcheck(|c, duel| i64::from(c.get_level(duel))),
        tp,
        HINTMSG_CONFIRM,
    )?;
    duel.confirm_cards(1 - tp, sg)?;
    duel.shuffle_hand(tp)?;
    a.e.set_label(duel, sg.len() as i64);
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    let target_count = a.e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, target_count, target_count, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, target_count, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    let c = a.e.get_handler(duel);
    for tc in tg {
        tc.negate_effects(duel, c, 0, true);
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_reason(duel, REASON_EFFECT) && c.is_location(duel, LOCATION_GRAVE)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
