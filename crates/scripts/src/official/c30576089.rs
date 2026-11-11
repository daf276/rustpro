// Blue-Eyes Jet Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 30576089;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

// A copy in the hand to Special Summon and one on the field to bounce, with Blue-Eyes White Dragons on the field and in the
// Graveyard, and Raigeki and an opponent's monster to destroy them and to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (12580477, HAND | OPPONENT),
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
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_target_range(duel, LOCATION_ONFIELD, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, e, c| c != e.get_handler(duel));
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, condition);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && (c.is_faceup(duel) && c.is_on_field(duel) || c.is_location(duel, LOCATION_GRAVE))
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
}

fn spcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, spcfilter, 1, None)) && condition(duel, a)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && (c.is_location(duel, LOCATION_GRAVE) && !a.eg.is_some_and(|eg| eg.is_contains(c)) || c.is_location(duel, LOCATION_HAND)));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
