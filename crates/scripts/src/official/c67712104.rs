// Marincess Crystal Heart
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 67712104;

pub const LISTED_SERIES: &[u16] = &[SET_MARINCESS];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(lc), sumtype, tp)
    }
}

// Link Summoned from WATER Normal Monsters (Psychic Kappa, Slime Toad), with Marincess Sea Star to discard against an attacker.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7892180, MZONE),
    (7892180, MZONE),
    (68638985, MZONE),
    (68638985, MZONE),
    (62886670, HAND),
    (62886670, HAND),
    (62886670, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_sequence(duel) > 4);
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_condition(duel, imcon);
    e2.set_target_card(duel, |duel, e, c| Some(c) == e.get_handler(duel).get_battle_target(duel));
    e2.set_value(duel, Value::Effect(imval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, indescond);
    e3.set_cost(duel, indescost);
    e3.set_target(duel, indestg);
    e3.set_operation(duel, indesop);
    c.register_effect(duel, e3);
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_monster_effect(duel) && te.get_owner_player(duel) != e.get_handler_player(duel)
}

fn imcon(duel: &Duel, _: &Args) -> bool {
    let ph = duel.get_current_phase();
    ph == PHASE_DAMAGE || ph == PHASE_DAMAGE_CAL
}

fn imval(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    e.get_handler(duel).get_battle_target(duel) != Some(re.get_owner(duel))
}

fn indescond(duel: &Duel, a: &Args) -> bool {
    let Some(at) = duel.get_attack_target() else {
        return false;
    };
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    at == c
        || at.is_controler(duel, tp)
            && at.is_faceup(duel)
            && at.is_set_card(duel, &[SET_MARINCESS])
            && c.get_linked_group(duel).contains(at)
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MARINCESS]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn indescost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, costfilter, 1, 1, REASON_COST)?;
    Ok(false)
}

fn indestg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(at) = duel.get_attack_target() {
        at.create_effect_relation(duel, a.e);
    }
    Ok(false)
}

fn indesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(at) = duel.get_attack_target().filter(|at| at.is_relate_to_effect(duel, e)) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        at.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
        e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e2.set_target_range(duel, 1, 0);
        e2.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        duel.register_effect(e2, tp);
    }
    Ok(())
}
