// Ultimate Leo Utopia Ray
use engine::Operation;

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 68679595;
pub const XYZ_NUMBER: u32 = 39;

pub const LISTED_NAMES: &[u32] = &[56840427];

// On the field with Xyz Import to attach Sangan, which Leo detaches to equip ZW - Ultimate Shield from the Deck, and more Sangan to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (18865703, DECK),
    (18865703, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(
        duel,
        EFFECT_FLAG_CANNOT_INACTIVATE + EFFECT_FLAG_CANNOT_NEGATE + EFFECT_FLAG_CANNOT_DISABLE,
    );
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, eqtg::<{ LOCATION_DECK | LOCATION_EXTRA }>);
    e1.set_operation(duel, eqop::<{ LOCATION_DECK | LOCATION_EXTRA }>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

// Whether a ZW card `c` has an equip effect that accepts `tc`; Ultimate Dragonic Utopia Ray reads the same effects.
pub(crate) fn filter(c: CardId, duel: &Duel, tc: CardId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ZW])
        && !c.is_forbidden(duel)
        && c.get_card_effect(duel, aux::ZW_EQUIP)
            .into_iter()
            .any(|te| matches!(te.get_value(duel), Value::ZwEquip(equipval) if equipval(duel, tc, c, tp)))
}

pub(crate) fn eqtg<const LOCATION: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|zw, duel| filter(zw, duel, c, tp), tp, LOCATION, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION as i32);
    Ok(false)
}

pub(crate) fn eqop<const LOCATION: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 || c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_matching_card(tp, |zw, duel| filter(zw, duel, c, tp), tp, LOCATION, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && let Some(eff) = tc.get_card_effect(duel, aux::ZW_EQUIP).into_iter().next()
        && let (Some(Operation::ZwEquip(equipop)), Some(linked)) = (eff.get_operation(duel), eff.get_label_object(duel))
    {
        equipop(duel, tc, linked, tp, c)?;
    }
    Ok(())
}

fn discfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ZW]) && c.get_original_type(duel) & TYPE_MONSTER != 0
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_equip_group(duel).is_exists(duel, discfilter, 1, None)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_disabled(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && !tc.is_disabled(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e2);
        if tc.is_immune_to_effect(duel, e1) || tc.is_immune_to_effect(duel, e2) {
            return Ok(());
        }
        duel.adjust_instantly(Some(tc));
        let atk = tc.get_attack(duel) / 2;
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e3.set_value(duel, atk);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e3);
    }
    Ok(())
}
