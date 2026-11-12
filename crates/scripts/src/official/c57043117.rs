// Giant Soldier of Steel
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 57043117;

// In the Extra Deck, Xyz Summoned with Level 3 Rocks (Giant Soldier of Stone, Dissolverock, Spherous Lady), with an attacker
// (Elfnote Lucina) on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13039848, MZONE),
    (40826495, MZONE),
    (52121290, MZONE),
    (52121290, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_ROCK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DEFCHANGE);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_monster_effect(duel) && te.get_owner(duel) != e.get_owner(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_code(duel, EFFECT_UPDATE_DEFENSE);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_CHANGE_DAMAGE);
        e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e2.set_target_range(duel, 1, 0);
        e2.set_value(duel, Value::ChangeDamage(damval));
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2, tp);
        let e3 = e2.clone_effect(duel);
        e3.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
        e3.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e3, tp);
    }
    Ok(())
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, rp: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT != 0 && rp != e.get_owner_player(duel) {
        0
    } else {
        val
    }
}
