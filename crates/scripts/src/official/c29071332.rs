// Armory Arm
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 29071332;

// On the field and in the Extra Deck, Synchro Summoned with Water Spirit (Level 1 Tuner) and Petit Angel (Level 3), with
// Elfnote Lucina to equip and attack the opponent's Sangan with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (487395, MZONE),
    (38142739, MZONE),
    (13597785, MZONE),
    (13597785, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP_ATTACK>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, 1000);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID + 1));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EVENT_BATTLE_DESTROYED);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_condition(duel, damcon);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID)
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END & !(RESET_TOFIELD | RESET_LEAVE), 0, 1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && duel.get_location_count(tp, LOCATION_SZONE) > 0) {
        duel.send_to_grave(c, REASON_RULE)?;
        return Ok(());
    }
    if duel.equip(tp, c, tc)? {
        c.register_flag_effect(duel, ID + 1, RESET_EVENT | RESETS_STANDARD, 0, 1);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID)
            && c.has_flag_effect(duel, ID + 1)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, tp, 0xff));
    }
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END & !(RESET_TOFIELD | RESET_LEAVE), 0, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bc = a.eg.and_then(|eg| eg.get_first());
    let eqc = c.get_equip_target(duel);
    c.has_flag_effect(duel, ID + 1)
        && bc.is_some_and(|bc| {
            eqc.is_some_and(|eqc| eqc.is_reason_card(duel, bc)) && bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel)
        })
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = a.eg.and_then(|eg| eg.get_first());
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    bc.create_effect_relation(duel, a.e);
    duel.set_target_player(1 - a.tp);
    let atk = bc.get_attack(duel);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, atk);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let bc = a.eg.and_then(|eg| eg.get_first());
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(()) };
    if bc.is_relate_to_effect(duel, a.e) {
        let atk = bc.get_attack(duel);
        if atk <= 0 {
            return Ok(());
        }
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        duel.damage(p, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}
