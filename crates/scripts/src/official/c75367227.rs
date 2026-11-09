// Ghostrick Alucard
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 75367227;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

// On the field and in the Extra Deck, with Xyz Import to attach the opponent's monsters; a Set Trap to destroy, and Ghostrick Shot
// in the GY to search; Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (25542642, SZONE | OPPONENT),
    (69809989, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, e, c| i32::from(tg(duel, e, c))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn tg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c != e.get_handler(duel) && (c.is_facedown(duel) || c.is_set_card(duel, &[SET_GHOSTRICK]))
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && desfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(desfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, desfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_facedown(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    let exception = Some(a.e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, exception)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
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
