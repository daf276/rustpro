// Mementotlan-Horned Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 55272555;

// In the hand and on the field, with three different Mementotlan monsters in the Graveyard, and Mementotlan Dark Blade and
// Tatsunootoshigo on the field to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (54550967, GRAVE),
    (81677154, GRAVE),
    (52918032, GRAVE),
    (18165869, MZONE),
    (81677154, MZONE),
    (54550967, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let g = duel.get_matching_group(
        |c, duel| c.is_monster(duel) && c.is_set_card(duel, &[SET_MEMENTO]),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && g.len() >= 3 && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_EFFECT) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card(duel, &[SET_MEMENTO]) && c.is_controler(duel, tp),
            1,
            None,
        ),
        false,
    )
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| c.is_faceup(duel) && c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        None,
    );
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 3, 3, &rescon));
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &rescon, tp, HINTMSG_DESTROY)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), 3, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
