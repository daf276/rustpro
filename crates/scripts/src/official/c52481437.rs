// Super Express Bullet Train
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 52481437;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with only Earth Machines on the field, attacking the opponent's Sangan, and in the Graveyard with a Machine to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (10859908, MZONE),
    (38916461, MZONE),
    (45945685, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_COST);
    e2.set_cost_condition(duel, |duel, e, _, tp, _| atcost(duel, e, tp));
    e2.set_operation(duel, atop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_operation(duel, regop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_condition(duel, thcon);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_EARTH)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    !g.is_empty() && g.filter_count(duel, cfilter, None) == g.len()
}

fn atcost(duel: &Duel, e: EffectId, tp: u8) -> bool {
    let c = e.get_handler(duel);
    duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, 2, Some(c.into()))
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.is_attack_cost_paid() != 2 && c.is_location(duel, LOCATION_MZONE) {
        let g = duel.get_matching_group(CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, Some(c.into()));
        let finishcon = |duel: &Duel, _: CardSet, _: CardSet, _: Option<CardId>| (duel.is_attack_cost_paid() == 0, false);
        let sg = aux::select_unselect_group_full(
            duel,
            g,
            0,
            2,
            &|_, _, _, _| (true, false),
            tp,
            HINTMSG_TOGRAVE,
            Some(&finishcon),
            false,
        )?;
        if sg.len() == 2 {
            duel.send_to_grave(sg, REASON_COST)?;
            duel.attack_cost_paid();
        } else {
            duel.attack_cost_paid_full(2);
        }
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
