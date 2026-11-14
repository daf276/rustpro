// Ancient Warriors - Virtuous Liu Xuan
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 40428851;

pub const LISTED_SERIES: &[u16] = &[SET_ANCIENT_WARRIORS];
pub const LISTED_NAMES: &[u32] = &[ID];

// Copies on the field to attack, with Elfnote Fortuna to attack into. The summon effect needs another Ancient Warriors monster in the
// Deck and none other is ported, so it is only reported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE), (85976588, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ANCIENT_WARRIORS])),
            a.e.get_owner_player(duel),
            LOCATION_MZONE,
            0,
            1,
            Some(a.e.get_handler(duel).into()),
        )
    });
    e1.set_value(duel, Value::Card(aux::imval1));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, ID);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > duel.get_field_group_count(a.tp, LOCATION_MZONE, 0)
    });
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, drcon);
    e3.set_target(duel, stock::draw_target::<1>);
    e3.set_operation(duel, stock::draw);
    c.register_effect(duel, e3);
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| costfilter(c, duel, tp), tp, LOCATION_ONFIELD | LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| costfilter(c, duel, tp),
        tp,
        LOCATION_ONFIELD | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ANCIENT_WARRIORS])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_DECK,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let (tp, handler) = (a.tp, a.e.get_handler(duel));
    let ancient_warrior = |c: CardId| c.is_set_card(duel, &[SET_ANCIENT_WARRIORS]);
    duel.get_attacker()
        .is_some_and(|atk| atk.is_controler(duel, tp) && atk != handler && ancient_warrior(atk))
        || duel
            .get_attack_target()
            .is_some_and(|b| b.is_controler(duel, tp) && b.is_faceup(duel) && b != handler && ancient_warrior(b))
}
