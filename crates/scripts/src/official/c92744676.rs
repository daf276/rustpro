// Zapper Shrimp
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 92744676;

// Synchro Summoned from Lunar Rabbit Omajinai and Key Mace (Level 1), with opponent's Traps to destroy and a copy on the field
// to be used as the Tuner of a Level 4 Synchro Summon (Armory Arm, Toxic Boxfish) with Doron.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (47643326, MZONE),
    (47643326, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (756652, MZONE),
    (29071332, EXTRA),
    (48882106, EXTRA),
    (53239672, SZONE | OPPONENT),
    (53239672, SZONE | OPPONENT),
    (1929294, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e2.set_target(duel, synchtg);
    e2.set_operation(duel, synchop);
    c.register_effect(duel, e2);
}

fn descostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, Some(c.get_equip_group(duel)))
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| descostfilter(c, duel, tp),
            tp,
            LOCATION_ONFIELD | LOCATION_HAND,
            0,
            1,
            Some(c.into()),
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| descostfilter(c, duel, tp),
        tp,
        LOCATION_ONFIELD | LOCATION_HAND,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn synchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn synchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_controler(duel, 1 - tp) || !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        duel.synchro_summon(tp, sc, Some(c.into()), None)?;
    }
    Ok(())
}
