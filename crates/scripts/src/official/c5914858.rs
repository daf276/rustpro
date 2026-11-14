// Junora the Power Patron of Tuning
use crate::prelude::*;

const ID: u32 = 5914858;

pub const LISTED_SERIES: &[u16] = &[SET_ELFNOTE];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 2;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }

    fn sub1(c: CardId, duel: &Duel, _: CardId, _: u64, tp: u8) -> bool {
        c.is_sequence(duel, 2) && c.is_controler(duel, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn negatable(c: CardId, duel: &Duel) -> bool {
    c.is_negatable(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(negatable, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(negatable, a.tp, 0, LOCATION_ONFIELD, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        tc.negate_effects(duel, c, 0, true);
    }
    Ok(())
}

fn cannotmatfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.get_column_group(duel)
        .is_exists(duel, |c, duel| elfnotefilter(c, duel, tp), 1, None)
}

fn elfnotefilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ELFNOTE]) && c.is_monster(duel) && c.is_faceup(duel) && c.is_controler(duel, tp)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cannotmatfilter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, None));
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(|c, duel| cannotmatfilter(c, duel, tp), tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        duel.hint_selection(tc);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
        e1.set_value(
            duel,
            Value::Material(|_, _, _, sumtype, _| {
                let allowed = [SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK];
                aux::cannotmatfilter(&allowed.map(u64::from), sumtype)
            }),
        );
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
