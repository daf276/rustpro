// Groza, Tyrant of Thunder
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 45420955;

// In the Extra Deck, Synchro Summoned with Zeta (Level 4 Tuner) and Grand Tiki Elder (Level 4 Fiend), with Fiends in the hand to discard and the opponent's Sangan to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (42026772, MZONE),
    (13676474, MZONE),
    (13676474, HAND),
    (13676474, HAND),
    (13676474, HAND),
    (26202165, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_FIEND, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, condition);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_disabled(duel)
}

fn dfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_discardable(duel)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && duel.is_main_phase()
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(disfilter(chkc, duel) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
            && duel.is_existing_matching_card(dfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.discard_hand(tp, dfilter, 1, 1, REASON_EFFECT | REASON_DISCARD)? == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if ((tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER)) && tc.is_relate_to_effect(duel, e) {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        super::c97268402::negate(duel, c, tc, EFFECT_FLAG_CANNOT_DISABLE);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && c.is_race(duel, RACE_FIEND) && c.is_controler(duel, tp) && c.is_previous_location(duel, LOCATION_HAND)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g1 = c.is_has_effect(duel, EFFECT_INDESTRUCTABLE_BATTLE).is_none();
    let g2 = c.is_has_effect(duel, EFFECT_INDESTRUCTABLE_EFFECT).is_none();
    let g3 = c.is_has_effect(duel, EFFECT_CANNOT_BE_EFFECT_TARGET).is_none();
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None) && (g1 || g2 || g3)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let b1 = c.is_has_effect(duel, EFFECT_INDESTRUCTABLE_BATTLE).is_none();
    let b2 = c.is_has_effect(duel, EFFECT_INDESTRUCTABLE_EFFECT).is_none();
    let b3 = c.is_has_effect(duel, EFFECT_CANNOT_BE_EFFECT_TARGET).is_none();
    let dtab: Vec<u64> = [(b1, 2), (b2, 3), (b3, 4)]
        .into_iter()
        .filter(|&(b, _)| b)
        .map(|(_, n)| aux::stringid(ID, n))
        .collect();
    let mut opt = duel.select_option(tp, &dtab)? + 1;
    if !(b1 && b2) {
        opt = 3;
    }
    if !(b1 && b3) {
        opt = 2;
    }
    if b1 && b3 && !b2 && opt == 2 {
        opt = 3;
    }
    if b2 && b3 && !b1 {
        opt += 1;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    if opt == 1 {
        e1.set_description(duel, 3000);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_owner_player(duel, tp);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
    } else if opt == 2 {
        e1.set_description(duel, 3060);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, Value::EffectPlayer(efilter));
    } else {
        e1.set_description(duel, 3061);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
        e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    }
    c.register_effect(duel, e1);
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, _: EffectId, rp: u8) -> bool {
    e.get_handler_player(duel) == 1 - rp
}
