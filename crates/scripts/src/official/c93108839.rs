// Number 58: Burner Visor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 93108839;
pub const XYZ_NUMBER: u32 = 58;

// On the field and in the Extra Deck (Xyz Summoned with two Level 4 Normal Monsters), with another Xyz Monster to equip to and the
// opponent's monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (63746411, MZONE),
    (549481, MZONE),
    (ID, EXTRA),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_target(duel).is_some());
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_BATTLE_DAMAGE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_condition(duel, damcon);
    e4.set_cost(duel, damcost);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ)
}

fn register_flag(duel: &mut Duel, c: CardId) {
    c.register_flag_effect(
        duel,
        ID,
        RESET_EVENT | RESETS_STANDARD & !(RESET_LEAVE | RESET_TOFIELD) | RESET_PHASE | PHASE_END,
        0,
        1,
    );
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
    register_flag(duel, c);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) || tc.is_controler(duel, 1 - tp) {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(());
    }
    if !duel.equip_full(tp, c, tc, false, false)? {
        return Ok(());
    }
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_EQUIP_LIMIT);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c))),
    );
    e0.set_label_object_group(duel, Some(tc.into()));
    e0.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e0);
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    register_flag(duel, c);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, true, false, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp && a.eg.and_then(|eg| eg.get_first()) == a.e.get_handler(duel).get_equip_target(duel)
}

fn damcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_target_param(500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 500);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}
