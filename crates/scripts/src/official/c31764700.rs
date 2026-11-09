// Yubel - The Ultimate Nightmare
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 31764700;

// On the field, with Sangans of the opponent's to fight.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_BATTLED);
    e3.set_operation(duel, batop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DAMAGE + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_DAMAGE_STEP_END);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    e4.set_label_object(duel, Some(e3));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e5);
}

fn batop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    match c.get_battle_target(duel) {
        Some(bc) if c.is_attack_pos(duel) => {
            let atk = bc.get_attack(duel);
            e.set_label(duel, atk.into());
            e.set_label_object_group(duel, Some(bc.into()));
        }
        _ => e.set_label_object_group(duel, None),
    }
    Ok(())
}

// The battled monster and the ATK recorded by the continuous effect.
fn recorded(duel: &Duel, e: EffectId) -> Option<(Option<CardId>, i64)> {
    let batop = e.get_label_object(duel)?;
    Some((
        batop.get_label_object_group(duel).and_then(|g| g.get_first()),
        batop.get_label(duel),
    ))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some((bc, atk)) = recorded(duel, a.e) else { return Ok(false) };
    if chk == 0 {
        return Ok(bc.is_some());
    }
    unported!(duel, bc.is_none());
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, atk as i32);
    if let Some(bc) = bc.filter(|bc| bc.is_relate_to_battle(duel)) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(bc.into()), 1, 0, 0);
    }
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some((bc, atk)) = recorded(duel, a.e) else { return Ok(()) };
    duel.damage(1 - a.tp, atk as u32, REASON_EFFECT)?;
    unported!(duel, bc.is_none());
    if let Some(bc) = bc.filter(|bc| bc.is_relate_to_battle(duel)) {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
