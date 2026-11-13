// Black-Winged Assault Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ADD_COUNTER;

const ID: u32 = 73218989;

pub const COUNTER_LIST: &[u16] = &[COUNTER_FEATHER];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_WINGED_DRAGON];
// Three in the Extra Deck, Synchro Summoned with Formula Synchron (a Level 2 Tuner Synchro Monster) and Blue-Eyes White Dragon, with
// the opponent's monsters to place Black Feather Counters from.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (50091196, MZONE),
    (50091196, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (14558127, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Message(MSG_ADD_COUNTER),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_FEATHER);
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    c.add_must_be_synchro_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1a.set_code(duel, EVENT_CHAINING);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1b.set_code(duel, EVENT_CHAIN_SOLVED);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && a.e.get_handler(duel).has_flag_effect(duel, 1)
    });
    e1b.set_operation(duel, damop);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_composite_type(duel, TYPE_SYNCHRO | TYPE_TUNER) || c.is_code(duel, CARD_BLACK_WINGED_DRAGON))
        && c.is_faceup(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, true)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    if sg.len() != 2 || duel.get_location_count_from_ex(tp, tp, Some(sg), e.get_handler(duel)) == 0 {
        return (false, false);
    }
    let mut cards = sg.into_iter();
    let (Some(a), Some(b)) = (cards.next(), cards.next()) else {
        return (false, false);
    };
    let tuner_synchro = |c: CardId| c.is_composite_type(duel, TYPE_SYNCHRO | TYPE_TUNER);
    let dragon = |c: CardId| c.is_code(duel, CARD_BLACK_WINGED_DRAGON);
    ((tuner_synchro(a) && dragon(b)) || (tuner_synchro(b) && dragon(a)), false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let (e, tp) = (a.e, c.get_controler(duel));
    let g = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, e, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let g = aux::select_unselect_group_full(
        duel,
        g,
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, e, tp),
        tp,
        HINTMSG_REMOVE,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.add_counter(duel, COUNTER_FEATHER, 1) {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.damage(1 - tp, 700, REASON_EFFECT)?;
    }
    Ok(())
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_counter(duel, COUNTER_FEATHER) >= 4 && c.is_releasable(duel));
    }
    duel.release(c, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(
        aux::TRUE,
        a.tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
