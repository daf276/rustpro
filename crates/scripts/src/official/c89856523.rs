// Brotherhood of the Fire Fist - Kirin
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 89856523;

pub const LISTED_SERIES: &[u16] = &[SET_FIRE_FORMATION, SET_FIRE_FIST];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_set_card_full(duel, &[SET_FIRE_FIST], Some(sc), sumtype, tp)
    }
}

// Brotherhood of the Fire Fist - Boar (Level 4 Tuner) with Buffalo for a Synchro Summon from the Extra Deck, and Fire Formation -
// Tenki in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (66762372, MZONE),
    (92572371, MZONE),
    (57103969, DECK),
    (57103969, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, setcon);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SYNCHRO
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FIRE_FORMATION]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.sset(tp, tc)?;
    }
    Ok(())
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel)
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(atkfilter, e.get_handler_player(duel), LOCATION_ONFIELD, 0, None) as i32 * -100
}
