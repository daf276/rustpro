// Psychic Omnibuster
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 70659412;
const DECLARED_TYPES: u64 = (ID as u64) << 16;
const ALL_TYPES: u32 = TYPE_MONSTER | TYPE_SPELL | TYPE_TRAP;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp)
    }
}

struct ReturnToHand;

impl aux::DelayedOperation for ReturnToHand {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(ag, PLAYER_NONE, REASON_EFFECT)?;
        Ok(())
    }
}

// In the Extra Deck, Synchro Summoned with Stinger the Electric Expeller (a Level 3 Tuner) and Mekk-Knight Avram (Level 4), with the
// opponent's Dark Hole and a full hand to chain with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (57649113, MZONE),
    (57649113, MZONE),
    (84754430, MZONE),
    (84754430, MZONE),
    (53129443, HAND | OPPONENT),
    (84754430, HAND | OPPONENT),
    (84754430, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        let location: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
        a.ep == 1 - a.tp && location & (LOCATION_MZONE | LOCATION_SZONE) > 0
    });
    e1.set_cost(duel, Cost::pay_lp::<2000>);
    e1.set_target(duel, looktg);
    e1.set_operation(duel, lookop);
    c.register_effect(duel, e1);
    aux::global_check(duel, ID, |duel| aux::add_values_reset(duel, reset));
}

fn reset(duel: &mut Duel) {
    duel.set_script_state(DECLARED_TYPES, 0);
    duel.set_script_state(DECLARED_TYPES | 1, 0);
}

fn looktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let key = DECLARED_TYPES | u64::from(tp);
    let declared = duel.get_script_state(key) as u32;
    if chk == 0 {
        return Ok(declared != ALL_TYPES && duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CARDTYPE);
    let effects = [
        (declared & TYPE_MONSTER == 0, DECLTYPE_MONSTER),
        (declared & TYPE_SPELL == 0, DECLTYPE_SPELL),
        (declared & TYPE_TRAP == 0, DECLTYPE_TRAP),
    ];
    let Some(op) = duel.select_effect(tp, &effects)? else {
        return Ok(false);
    };
    let typ = 1 << (op - 1);
    duel.set_script_state(key, i64::from(declared | typ));
    duel.set_target_param(typ);
    Ok(false)
}

fn lookop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, 0, LOCATION_HAND) == 0 {
        return Ok(());
    }
    let hand = duel.get_field_group(tp, 0, LOCATION_HAND);
    let Some(rc) = hand.random_select(duel, tp, 1).get_first() else {
        return Ok(());
    };
    duel.confirm_cards(tp, rc)?;
    duel.shuffle_hand(1 - tp)?;
    let typ: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if !rc.is_type(duel, typ) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        let desc = match typ {
            TYPE_MONSTER => 3010,
            TYPE_SPELL => 3011,
            _ => 3012,
        };
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_description(duel, desc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e1.set_label(duel, i64::from(typ));
        e1.set_value(duel, Value::Effect(|duel, e, te| te.is_active_type(duel, e.get_label(duel) as u32)));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    if rc.is_able_to_remove(duel) {
        duel.break_effect()?;
        aux::remove_until::<ReturnToHand>(duel, rc, POS_FACEUP, REASON_EFFECT | REASON_TEMPORARY, PHASE_END, ID, e, tp)?;
    }
    Ok(())
}
