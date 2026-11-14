// Galaxy-Eyes Cipher X Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 28346136;

pub const LISTED_SERIES: &[u16] = &[SET_CIPHER_DRAGON];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field and in the Extra Deck, Xyz Summoned on a Galaxy-Eyes Cipher Dragon, with another in the Graveyard to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (18963306, MZONE),
    (18963306, MZONE),
    (18963306, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 2;
    const MAXCT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, cannottgtg);
    e1.set_operation(duel, cannottgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, texsptg);
    e2.set_operation(duel, texspop);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_set_card_full(duel, &[SET_CIPHER_DRAGON], Some(lc), SUMMON_TYPE_XYZ as u64, tp) && c.is_faceup(duel)
}

fn cannottgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_faceup(duel) && !c.has_flag_effect(duel, ID)
}

fn cannottgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(cannottgfilter, a.tp, LOCATION_MZONE, 0, 1, None))
}

fn cannottgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(cannottgfilter, tp, LOCATION_MZONE, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let ct = if duel.is_turn_player(tp) { 2 } else { 1 };
    for tc in g {
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, ct);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3061);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
        e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, ct);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn texfilter(c: CardId, duel: &Duel) -> bool {
    c.is_rank_below(duel, 9) && c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_XYZ) && c.is_able_to_extra(duel)
}

fn texsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(texfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn texspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, texfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    duel.hint_selection(sc);
    if !(duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_EXTRA)) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if !(pg.is_empty() || (pg.len() == 1 && pg.contains(c))) {
        return Ok(());
    }
    if c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
        && c.is_controler(duel, tp)
        && !c.is_immune_to_effect(duel, e)
        && c.is_can_be_xyz_material(duel, Some(sc), tp, REASON_XYZ | REASON_MATERIAL)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), sc) > 0
        && sc.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.break_effect()?;
        sc.set_material(duel, Some(c.into()));
        duel.overlay(sc, c)?;
        if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
            sc.complete_procedure(duel);
        }
    }
    Ok(())
}
