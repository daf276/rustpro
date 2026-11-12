// Number 39: Utopia Double
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 62517849;
const DOUBLE_OR_NOTHING: u32 = 94770493;
pub const LISTED_NAMES: &[u32] = &[DOUBLE_OR_NOTHING];
pub const XYZ_NUMBER: u32 = 39;

// In the Extra Deck, Xyz Summoned with two Level 4 Normal Monsters, with Double or Nothing! in the Deck and Number 39: Utopia to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (84013237, EXTRA),
    (84013237, EXTRA),
    (DOUBLE_OR_NOTHING, DECK),
    (DOUBLE_OR_NOTHING, DECK),
    (5818798, MZONE),
    (5818798, MZONE),
    (5818798, MZONE),
    (5818798, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_END_PHASE);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, DOUBLE_OR_NOTHING) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_UTOPIA])
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok((pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
            && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    duel.break_effect()?;
    aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if !(c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.is_controler(duel, tp) && !c.is_immune_to_effect(duel, e)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = sg.get_first() else { return Ok(()) };
    duel.break_effect()?;
    sc.set_material(duel, Some(c.into()));
    duel.overlay(sc, c)?;
    if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    sc.complete_procedure(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3207);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sc.register_effect(duel, e1);
    let atk = sc.get_text_attack(duel) * 2;
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_SET_ATTACK);
    e2.set_value(duel, atk);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sc.register_effect(duel, e2);
    Ok(())
}
