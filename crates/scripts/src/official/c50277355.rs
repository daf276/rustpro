// Cross-Sheep
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 50277355;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property(duel, sg, &|c, duel| c.get_codes_full(duel, Some(lc), sumtype, tp))
    }
}

// In the Extra Deck, Link Summoned from two different monsters next to a Fusion (Garura), a Synchro (Magical Android) and
// an Xyz Monster (Dark Rebellion Xyz Dragon), then Monster Reborn on a Level 4 or lower monster in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (4035199, MZONE),
    (11765832, MZONE),
    (43385557, MZONE),
    (16195942, MZONE),
    (26202165, GRAVE),
    (83764718, HAND),
    (83764718, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES + CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        aux::zptgroupcon(duel, a.eg.unwrap_or_default(), aux::TRUE, a.e.get_handler(duel))
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e1);
}

fn linked_exists(duel: &Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool) -> bool {
    a.e.get_handler(duel).get_linked_group(duel).is_exists(duel, filter, 1, None)
}

fn rittg(duel: &mut Duel, a: &Args, chk: i32) -> bool {
    let tp = a.tp;
    if chk == 0 {
        return linked_exists(duel, a, aux::faceup_filter(CardId::is_ritual_monster)) && duel.is_player_can_draw(tp, 2);
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 2);
    false
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn fustg(duel: &mut Duel, a: &Args, chk: i32) -> bool {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return linked_exists(duel, a, aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_FUSION)))
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    false
}

fn syntg(duel: &mut Duel, a: &Args, chk: i32) -> bool {
    if chk == 0 {
        return linked_exists(duel, a, aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_SYNCHRO)))
            && duel.is_existing_matching_card(CardId::is_faceup, a.tp, LOCATION_MZONE, 0, 1, None);
    }
    false
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32) -> bool {
    if chk == 0 {
        return linked_exists(duel, a, aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_XYZ)))
            && duel.is_existing_matching_card(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, 1, None);
    }
    false
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(rittg(duel, a, 0) || fustg(duel, a, 0) || syntg(duel, a, 0) || xyztg(duel, a, 0));
    }
    for tg in [rittg, fustg, syntg, xyztg] {
        if tg(duel, a, 0) {
            tg(duel, a, 1);
        }
    }
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 1, tp, 2);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 1, tp, 2);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn ritop(duel: &mut Duel, tp: u8) -> Script<()> {
    if duel.draw(tp, 2, REASON_EFFECT)? == 2 {
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 2, 2, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn fusop(duel: &mut Duel, e: EffectId, tp: u8) -> Script<()> {
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn updateop(duel: &mut Duel, a: &Args, location1: u32, location2: u32, amt: i32) {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, location1, location2, None);
    for tc in g {
        tc.update_attack(duel, amt, None, Some(a.e.get_handler(duel)), None);
    }
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if rittg(duel, a, 0) {
        ritop(duel, tp)?;
    }
    if fustg(duel, a, 0) {
        duel.break_effect()?;
        fusop(duel, e, tp)?;
    }
    if syntg(duel, a, 0) {
        duel.break_effect()?;
        updateop(duel, a, LOCATION_MZONE, 0, 700);
    }
    if xyztg(duel, a, 0) {
        duel.break_effect()?;
        updateop(duel, a, 0, LOCATION_MZONE, -700);
    }
    Ok(())
}
