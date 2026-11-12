// Melffy Godmommy
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 5833929;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, _, _, _| c.is_race(duel, RACE_BEAST));
    const LV: Option<u32> = Some(2);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(altxyzmatfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        Ok(duel
            .register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1)
            .is_some())
    }
}

// In the Extra Deck, Xyz Summoned over Melffy Mommy, with Melffy Tag in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (76833149, MZONE),
    (76833149, MZONE),
    (37256135, DECK),
    (37256135, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, Cost::detach_from_self::<5, 5>);
    e3.set_target(duel, rthtg);
    e3.set_operation(duel, rthop);
    c.register_effect(duel, e3);
}

fn altxyzmatfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_faceup(duel) && c.is_summon_code(duel, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp, &[76833149])
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MELFFY]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = e
        .get_handler(duel)
        .get_overlay_group(duel)
        .filter(duel, |c, duel| c.is_set_card(duel, &[SET_MELFFY]), None);
    (g.get_sum(duel, |c, duel| c.get_attack(duel).into()) + g.get_sum(duel, |c, duel| c.get_defense(duel).into())) as i32
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_able_to_hand, a.tp, 0, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
