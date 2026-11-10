// Sengenjin Wakes from a Millennium
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 38775407;

pub const LISTED_NAMES: &[u32] = &[CARD_MILLENNIUM_CROSS];
// In the hand to place and Special Summon, Millennium Ankh in the hand to reveal, and Millennium Shield in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_MILLENNIUM_CROSS, HAND), (32012841, DECK), (32012841, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    c.register_effect(duel, e1);
    add_placement_effects::<ID>(duel, c);
}

// Shared with the other Millennium monsters that place themselves as a Continuous Spell and Special Summon themselves.
pub fn add_placement_effects<const CODE: u32>(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(CODE, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, CODE);
    e2.set_target(duel, |duel, a, chk, _| {
        Ok(chk == 0 && duel.get_location_count(a.tp, LOCATION_SZONE) > 0)
    });
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(CODE, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (CODE, 1));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_continuous_spell(duel));
    e3.set_cost(duel, spcost::<CODE>);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop::<CODE>);
    c.register_effect(duel, e3);
}

pub fn treat_as_continuous_spell(duel: &mut Duel, source: CardId, c: CardId) {
    let e1 = duel.create_effect(source);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
    c.register_effect(duel, e1);
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    if duel.move_to_field(c, tp, tp, LOCATION_SZONE, POS_FACEUP, true)? {
        treat_as_continuous_spell(duel, c, c);
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MILLENNIUM_CROSS) && !c.is_public(duel)
}

fn spcost<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let b1 = duel.check_lp_cost(tp, 2000);
    let b2 = duel.is_existing_matching_card(spcostfilter, tp, LOCATION_HAND, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = if b1 && b2 {
        duel.select_effect(tp, &[(b1, aux::stringid(CODE, 2)), (b2, aux::stringid(CODE, 3))])?
    } else {
        Some(if b1 { 1 } else { 2 })
    };
    if op == Some(1) {
        duel.pay_lp_cost(tp, 2000)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(tp, spcostfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
    }
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    match CODE {
        ID => c.is_set_card(duel, &[SET_MILLENNIUM]) && c.is_monster(duel) && c.is_able_to_hand(duel),
        1164211 => c.is_code(duel, CARD_MILLENNIUM_CROSS) && c.is_able_to_hand(duel),
        _ => c.is_code(duel, 63017368) && c.is_able_to_hand(duel),
    }
}

fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(thfilter::<CODE>, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(CODE, 4))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter::<CODE>, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.send_to_hand(g, tp, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
