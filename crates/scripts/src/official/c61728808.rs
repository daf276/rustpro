// Heavymetalfoes Amalgam
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 61728808;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_METALFOES], Some(lc), sumtype, tp)
    }
}

// Metalfoes Silverd to Link Summon with, to equip to and to destroy, and the opponent's Raigeki to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (7868571, MZONE),
    (7868571, MZONE),
    (7868571, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_EFFECT)
}

fn spfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_METALFOES]) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let metalfoes = |c: CardId, duel: &Duel| c.is_set_card(duel, &[SET_METALFOES]);
    let b1 = duel.is_existing_target(metalfoes, tp, LOCATION_MZONE, 0, 1, None) && duel.get_location_count(tp, LOCATION_SZONE) > 0;
    let b2 = duel.is_existing_target(|c, duel| spfilter(c, duel, ft), tp, LOCATION_ONFIELD, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let mut op = 0;
    if b1 && b2 {
        op = duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?;
    } else if b2 {
        op = 1;
    }
    e.set_label(duel, i64::from(op));
    if op == 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        duel.select_target(tp, metalfoes, tp, LOCATION_MZONE, 0, 1, 1, None)?;
        e.set_category(duel, CATEGORY_EQUIP);
        duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, 0, 0);
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_target(tp, |c, duel| spfilter(c, duel, ft), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        e.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(e.get_handler(duel).into()), 1, 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let label = e.get_label(duel);
    if label == 0 && duel.get_location_count(tp, LOCATION_SZONE) > 0 && !tc.is_facedown(duel) {
        duel.equip(tp, c, tc)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_value(duel, 1000);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    } else if label == 1 && duel.destroy(tc, REASON_EFFECT)? != 0 && c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
