// Summon Sorceress
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 61665245;

// In the Extra Deck, Link Summoned with two Dark Magicians (Spellcasters), with another in the hand and the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        aux::check_same_property(duel, sg, &|c, duel| c.get_race(duel)).0
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn hspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, 1 - tp);
    if chk == 0 {
        return Ok(
            duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) > 0
                && duel.is_existing_matching_card(|c, duel| hspfilter(c, duel, e, tp, zone), tp, LOCATION_HAND, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn dspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, race: u64) -> bool {
    c.is_race(duel, race) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let zone = c.get_linked_zone_full(duel, 1 - tp);
    if duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let hsc = duel.select_matching_card(tp, |c, duel| hspfilter(c, duel, e, tp, zone), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let hsc = hsc.get_first();
    if let Some(hsc) = hsc
        && duel.special_summon_step_full(hsc, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE, zone)?
    {
        hsc.negate_effects(duel, c, 0, false);
    }
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    let race = hsc.map_or(0, |hsc| hsc.get_race(duel));
    if !(duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| dspfilter(c, duel, e, tp, race), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?)
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let dsc = duel.select_matching_card(tp, |c, duel| dspfilter(c, duel, e, tp, race), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(dsc) = dsc.get_first() else { return Ok(()) };
    duel.break_effect()?;
    if duel.special_summon_step(dsc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? {
        dsc.negate_effects(duel, c, 0, false);
    }
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_label(duel, dsc.get_original_race(duel) as i64);
    e1.set_target_card(duel, |duel, e, c| !c.is_original_race(duel, e.get_label(duel) as u64));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
