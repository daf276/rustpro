// Binary Sorceress
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 79016563;
const HONEYBOT: u32 = 34472920;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        matfilter(c, duel, lc, sumtype, tp)
    }
}

// On the field between Honeybots (Left and Right arrows, so mutually linked), with weak monsters of the opponent to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (HONEYBOT, MZONE),
    (HONEYBOT, MZONE),
    (HONEYBOT, MZONE),
    (HONEYBOT, MZONE),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, reccon);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, atkcon);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
}

fn reccon(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_mutual_linked_group(duel);
    let Some(tc) = a.eg.and_then(|eg| eg.get_first()) else {
        return false;
    };
    a.ep != a.tp && lg.is_contains(tc) && tc.get_battle_target(duel).is_some()
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(a.ev);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, a.ev as i32);
    Ok(false)
}

fn recop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    aux::stat_change_damage_step_condition(duel, a) && a.e.get_handler(duel).get_mutual_linked_group_count(duel) >= 2
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g1 = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let hc = g1.and_then(|g| g.get_first()).map(CardSet::from);
    e.set_label_object_group(duel, hc);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, hc)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(hc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let Some(tc) = g.iter().find(|&c| c != hc) else { return Ok(()) };
    if hc.is_faceup(duel) && hc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let atk = hc.get_attack(duel);
        let half = (f64::from(atk) / 2.0).round() as i32;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, half);
        hc.register_effect(duel, e1);
        if !hc.is_immune_to_effect(duel, e1) {
            let e2 = duel.create_effect(e.get_handler(duel));
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_UPDATE_ATTACK);
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            e2.set_value(duel, half);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
