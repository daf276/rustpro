// Angelechy Castellan
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 101402090;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field facing monsters and Set cards of the opponent's to banish. Its effects as a Continuous Spell need Angelechy cards
// that place it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_MOVE);
    e2.set_range(duel, LOCATION_STZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, tgcon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3a.set_code(duel, EVENT_CHAINING);
    e3a.set_range(duel, LOCATION_STZONE);
    e3a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_continuous_spell(duel));
    e3a.set_operation(duel, |duel, a| {
        a.e.get_handler(duel)
            .register_flag_effect(duel, ID + u32::from(a.ep) * 100, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1);
        Ok(())
    });
    c.register_effect(duel, e3a);
    let e3b = duel.create_effect(c);
    e3b.set_type(duel, EFFECT_TYPE_FIELD);
    e3b.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3b.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3b.set_range(duel, LOCATION_STZONE);
    e3b.set_target_range(duel, 1, 1);
    e3b.set_condition(duel, |duel, a| a.e.get_handler(duel).is_continuous_spell(duel));
    e3b.set_value(
        duel,
        Value::EffectPlayer(|duel, e, _, rp| e.get_handler(duel).has_flag_effect(duel, ID + u32::from(rp) * 100)),
    );
    c.register_effect(duel, e3b);
}

fn normalizeseq(seq: u32) -> i64 {
    match seq {
        5 => 1,
        6 => 3,
        seq => i64::from(seq),
    }
}

fn banfilter(c: CardId, duel: &Duel, seq: i64) -> bool {
    if !c.is_able_to_remove(duel) {
        return false;
    }
    let dist = (seq - normalizeseq(c.get_sequence(duel))).abs();
    if c.is_location(duel, LOCATION_MZONE) {
        return dist == 1;
    }
    dist == 2
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let seq = 4 - normalizeseq(a.e.get_handler(duel).get_sequence(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_MZONE | LOCATION_STZONE)
            && banfilter(chkc, duel, seq));
    }
    let filter = |c: CardId, duel: &Duel| banfilter(c, duel, seq);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE | LOCATION_STZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE | LOCATION_STZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.eg.is_some_and(|eg| eg.is_contains(c)) && !c.is_previous_location(duel, LOCATION_SZONE)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
