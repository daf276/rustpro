// Repair Genex Controller
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 8173184;

pub const LISTED_SERIES: &[u16] = &[SET_GENEX];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_GENEX], Some(lc), sumtype, tp) && c.is_level_below(duel, 4)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// Link Summoned from Genex Controller, with Genex monsters in the Graveyard to add and in the hand to Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (68505803, MZONE),
    (68505803, MZONE),
    (68505803, GRAVE),
    (68505803, GRAVE),
    (68505803, HAND),
    (16828633, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| nsconfilter(c, duel, a.tp), 1, None)
    });
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GENEX]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_GRAVE)
}

fn nsconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_set_card(duel, &[SET_GENEX]) && !c.is_reason(duel, REASON_DRAW)
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GENEX]) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, nsfilter, LOCATION_HAND | LOCATION_MZONE, CATEGORY_SUMMON)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.summon(tp, tc, true, None)?;
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, st, _, _, proc_eff| {
        c.is_location(duel, LOCATION_EXTRA) && (st & SUMMON_TYPE_SYNCHRO != SUMMON_TYPE_SYNCHRO || proc_eff.is_none())
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check(duel, c, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM);
    e2.set_target_range(duel, LOCATION_MZONE | LOCATION_HAND, 0);
    e2.set_target_card(duel, |duel, _, c| {
        !(c.is_set_card(duel, &[SET_GENEX]) && c.is_type(duel, TYPE_TUNER))
    });
    e2.set_synchro_material_operation(duel, synop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}

fn gtfilter(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_GENEX], Some(sc), SUMMON_TYPE_SYNCHRO.into(), tp)
        && c.is_type_full(duel, TYPE_TUNER, Some(sc), SUMMON_TYPE_SYNCHRO.into(), tp)
}

fn synop(duel: &Duel, e: EffectId, tg: CardSet, ntg: CardSet, _: CardSet, _: u32, sc: CardId, tp: u8) -> (bool, bool) {
    let genex = |c: CardId, duel: &Duel| c.is_set_card_full(duel, &[SET_GENEX], Some(sc), SUMMON_TYPE_SYNCHRO.into(), tp);
    let ok = e.get_handler_player(duel) == 1 - tp
        || tg.is_exists(duel, genex, 1, None)
        || ntg.is_exists(duel, |c, duel| gtfilter(c, duel, sc, tp), 1, None);
    (ok, false)
}
