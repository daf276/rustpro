// Digital Bug Corebage
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 58600555;

// On the field with Petit Moth (an Insect) in the GY to attach, and the opponent's Sangan and Elfnote Lucina to change position. No Level 5 LIGHT Insect
// and no Rank 3 or 4 Insect Xyz Monster with two materials is ported, so the Xyz Summon is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (58192742, GRAVE),
    (58192742, GRAVE),
    (58192742, GRAVE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(mfilter);
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 2);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, mc: CardId) -> bool {
        mc.check_remove_overlay_card(duel, tp, 2, REASON_COST)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, mc: CardId) -> Script<bool> {
        mc.remove_overlay_card(duel, tp, 2, 2, REASON_COST)?;
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_code(duel, EVENT_CHANGE_POS);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_condition(duel, condition);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let rank = c.get_rank(duel);
    c.is_faceup(duel)
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && (rank == 3 || rank == 4)
        && c.is_race_full(duel, RACE_INSECT, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
}

fn mfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_INSECT, Some(xyz), sumtype, tp) && c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(xyz), sumtype, tp)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_DEFENSE) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    let (np, pp) = (c.get_position(duel), c.get_previous_position(duel));
    (np < 3 && pp > 3) || (pp < 3 && np > 3)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, cfilter, 1, None))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_type(duel, TYPE_XYZ)
            && duel.is_existing_matching_card(|c, duel| c.is_race(duel, RACE_INSECT), a.tp, LOCATION_GRAVE, 0, 1, None));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let filter = aux::necro_valley_filter(|c, duel| c.is_race(duel, RACE_INSECT));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.overlay(c, g)?;
    }
    Ok(())
}
