// Light End Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 25132288;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(sc), sumtype, tp)
        })
    }
}

// In the Extra Deck, Synchro Summoned with Junk Synchron (a Level 3 Tuner) and Orion the Battle King (Level 5 LIGHT), against the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (63977008, MZONE),
    (63977008, MZONE),
    (2971090, MZONE),
    (2971090, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition_mut(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let tc = c.get_battle_target(duel);
    a.e.set_label_object_group(duel, tc.map(CardSet::from));
    tc.is_some_and(|tc| {
        c.get_attack(duel) >= 500
            && c.get_defense(duel) >= 500
            && tc.is_faceup(duel)
            && (tc.get_attack(duel) > 0 || tc.get_defense(duel) > 0)
    })
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        tc.create_effect_relation(duel, a.e);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.get_attack(duel) >= 500 && c.get_defense(duel) >= 500 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -500);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        c.register_effect(duel, e2);
        if tc.is_relate_to_battle(duel)
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
            && c.is_has_effect(duel, EFFECT_REVERSE_UPDATE).is_none()
        {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
            e3.set_code(duel, EFFECT_UPDATE_ATTACK);
            e3.set_value(duel, -1500);
            tc.register_effect(duel, e3);
            let e4 = e3.clone_effect(duel);
            e4.set_code(duel, EFFECT_UPDATE_DEFENSE);
            tc.register_effect(duel, e4);
        }
    }
    Ok(())
}
