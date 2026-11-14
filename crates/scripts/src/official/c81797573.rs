// Angelechy Enlisted
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 81797573;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field, to banish beside it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_CONTROL_CHANGED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn get_adjacent_zones(duel: &Duel, c: CardId) -> u32 {
    if !c.is_location(duel, LOCATION_MZONE) {
        return 0;
    }
    let seq = match c.get_sequence(duel) {
        5 => 1,
        6 => 3,
        seq => seq,
    };
    let mut zones = 0;
    if seq > 0 {
        zones = 1 << (5 - seq);
    }
    if seq < 4 {
        zones |= 1 << (3 - seq);
    }
    zones
}

fn banfilter(c: CardId, duel: &Duel, tp: u8, zones: u32) -> bool {
    c.is_monster(duel)
        && c.is_controler(duel, 1 - tp)
        && c.is_able_to_remove(duel)
        && duel.get_mzone_count_full(1 - tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, zones) > 0
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zones = get_adjacent_zones(duel, c);
    let cg = c.get_column_group_full(duel, 1, 1).difference(c.get_column_group(duel)).filter(
        duel,
        |tc, duel| banfilter(tc, duel, tp, zones),
        Some(c.into()),
    );
    if let Some(chkc) = chkc {
        return Ok(cg.is_contains(chkc));
    }
    let targetable = |tc: CardId, duel: &Duel| tc.is_can_be_effect_target(duel, e);
    if chk == 0 {
        return Ok(cg.is_exists(duel, targetable, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = cg.filter_select(duel, tp, targetable, 1, 1, None)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zones = get_adjacent_zones(duel, c);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && (tc.is_type(duel, TYPE_TOKEN) || tc.is_location(duel, LOCATION_REMOVED))
        && c.is_relate_to_effect(duel, e)
        && duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_CONTROL, zones) > 0
    {
        duel.break_effect()?;
        duel.get_control_full(c, 1 - tp, 0, 0, zones, tp)?;
    }
    Ok(())
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, c.get_owner(duel), LOCATION_EXTRA as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY])
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_EXTRA)
    {
        let owner = c.get_owner(duel);
        duel.hint(HINT_SELECTMSG, owner, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(owner, |c, duel| spfilter(c, duel, e, owner), owner, LOCATION_EXTRA, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, owner, owner, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
