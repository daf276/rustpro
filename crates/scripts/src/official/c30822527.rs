// Decode Talker Extended
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 30822527;

pub const LISTED_NAMES: &[u32] = &[1861629];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Link Summoned from Effect Monsters (Reflect Bounder, Hyper Hammerhead).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2851070, MZONE),
    (2851070, MZONE),
    (2671330, MZONE),
    (2671330, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1861629);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_BATTLE_DESTROYED);
    e3.set_condition(duel, regcon);
    e3.set_operation(duel, regop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_condition(duel, regcon2);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e5.set_code(duel, EVENT_CUSTOM + ID);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_condition(duel, racon);
    e5.set_target(duel, ratg);
    e5.set_operation(duel, raop);
    c.register_effect(duel, e5);
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    e.get_handler(duel)
        .get_linked_group(duel)
        .filter_count(duel, CardId::is_monster, None) as i32
        * 500
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    let mut seq = c.get_previous_sequence(duel);
    if !c.is_previous_controler(duel, tp) {
        seq += 16;
    }
    c.is_previous_location(duel, LOCATION_MZONE) && zone >> seq & 1 != 0
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let (tp, zone) = (a.tp, a.e.get_handler(duel).get_linked_zone(duel));
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp, zone), 1, None))
}

fn regcon2(duel: &Duel, a: &Args) -> bool {
    let (tp, zone) = (a.tp, a.e.get_handler(duel).get_linked_zone(duel));
    a.eg.is_some_and(|eg| {
        eg.is_exists(
            duel,
            |c, duel| !c.is_reason(duel, REASON_BATTLE) && cfilter(c, duel, tp, zone),
            1,
            None,
        )
    })
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.raise_single_event(a.e.get_handler(duel), EVENT_CUSTOM + ID, Some(a.e), 0, a.tp, 0, 0)?;
    Ok(())
}

fn racon(duel: &Duel, a: &Args) -> bool {
    duel.get_turn_player() == a.tp && duel.is_battle_phase()
}

fn ratg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && a.e.get_handler(duel).get_effect_count(duel, EFFECT_EXTRA_ATTACK) == 0)
}

fn raop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
