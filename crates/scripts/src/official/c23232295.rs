// Battlin' Boxer Lead Yoke
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 23232295;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> =
        Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_BATTLIN_BOXER], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// Other Boxer Xyz Monsters whose detach cost raises the ATK, and the opponent's Raigeki to destroy them.
// The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (59627393, MZONE), (46804536, MZONE), (12580477, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    duel.enable_global_flag(GLOBALFLAG_DETACH_EVENT);
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    e1.set_value(duel, Value::Card(repval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EVENT_DETACH_MATERIAL);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_set_card(duel, &[SET_BATTLIN_BOXER])
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    if c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT) && duel.select_effect_yes_no_full(tp, c, 96)? {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
        let g = a.eg.unwrap_or_default().filter(duel, |c, duel| repfilter(c, duel, tp), None);
        if g.len() == 1 {
            e.set_label_object_group(duel, g.get_first().map(Into::into));
        } else {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
            let cg = g.select(duel, tp, 1, 1, None)?;
            e.set_label_object_group(duel, cg.get_first().map(Into::into));
        }
        return Ok(true);
    }
    Ok(false)
}

fn repval(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && a.e.get_handler(duel).is_relate_to_effect(duel, a.e))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e1.set_value(duel, 800);
        c.register_effect(duel, e1);
    }
    Ok(())
}
