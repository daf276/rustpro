// Number 45: Crumble Logos the Prophet of Demolition
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 29208536;
pub const XYZ_NUMBER: u32 = 45;

// On the field with Xyz Import to attach Sangan, which Logos detaches to negate the other Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_card_target_count(duel) > 0);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        c.set_card_target(duel, tc);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_condition(duel, disablecon);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn disablecon(duel: &Duel, a: &Args) -> bool {
    a.e.get_owner(duel).is_has_card_target(duel, a.e.get_handler(duel))
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let codes: Vec<u32> = e
        .get_handler(duel)
        .get_card_target(duel)
        .iter()
        .map(|tc| tc.get_code(duel))
        .collect();
    codes.iter().any(|&code| re.get_handler(duel).is_code(duel, code))
}
