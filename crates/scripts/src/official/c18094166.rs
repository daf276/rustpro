// Vision HERO Faris
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 18094166;

pub const LISTED_SERIES: &[u16] = &[SET_HERO, SET_VISION_HERO];
// In the hand with an Elemental HERO Stratos to discard, and Vision HERO Vyon in the Deck to place in the Spell & Trap Zone.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (40044918, HAND), (27780618, DECK)];
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
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_HERO]) && c.is_discardable(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand_full(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD, Some(c.into()))?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VISION_HERO]) && c.is_monster(duel) && !c.is_forbidden(duel) && !c.is_code(duel, ID)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk != 0
        || (duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None) && duel.get_location_count(tp, LOCATION_SZONE) > 0))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
        e1.set_value(duel, (TYPE_TRAP + TYPE_CONTINUOUS) as i32);
        tc.register_effect(duel, e1);
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, splimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, e.get_handler(duel), tp, |duel, _, c| {
        !c.is_original_set_card(duel, &[SET_HERO])
    });
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    !c.is_set_card(duel, &[SET_HERO]) && c.is_location(duel, LOCATION_EXTRA)
}
