// Cyberdark Horn
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 41230939;

struct Equipping;

impl CyberdarkEquip for Equipping {
    const MANDATORY: bool = true;

    fn f(c: CardId, duel: &Duel) -> bool {
        eqfilter(c, duel)
    }

    fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
        equipop(duel, c, e, tp, tc)
    }
}

// In the hand with Cyberdark Claws in the Graveyard to equip.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (82562802, GRAVE), (82562802, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, Cyberdark::equip_target::<Equipping>);
    e1.set_operation(duel, Cyberdark::equip_operation::<Equipping>);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 3) && c.is_race(duel, RACE_DRAGON)
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    let atk = tc.get_text_attack(duel).max(0);
    if !c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, None)? {
        return Ok(());
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_property(duel, EFFECT_FLAG_OWNER_RELATE + EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e2.set_value(duel, atk);
    tc.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_DESTROY_SUBSTITUTE);
    e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e3.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    tc.register_effect(duel, e3);
    Ok(())
}
