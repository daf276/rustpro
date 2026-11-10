// Muckraker From the Underworld
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 71607202;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Sangans (Fiend Effect Monsters) with another on the field to Tribute and one in the
// Graveyard to Special Summon; the opponent's Smashing Ground and Fissure destroy one monster to test the replacement.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, GRAVE),
    (26202165, GRAVE),
    (97169186, HAND | OPPONENT),
    (66788016, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e0.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_status(duel, STATUS_SPSUMMON_TURN) && c.is_link_summoned(duel)
    });
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    e1.set_operation(duel, repop);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn trbfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_race(duel, RACE_FIEND)
        && c.is_releasable_by_effect(duel)
        && !c.is_immune_to_effect(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED | STATUS_BATTLE_DESTROYED)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_existing_matching_card(|c, duel| trbfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, Some(eg)));
    }
    if !duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let tg = duel.select_matching_card(tp, |c, duel| trbfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, 1, Some(eg))?;
    duel.set_target_card(tg);
    Ok(true)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 1 - a.tp, ID.into());
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc {
        duel.release(tc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c: CardId, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
            tp,
            LOCATION_HAND,
            0,
            1,
            None,
        ) && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    if duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)? > 0
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e))
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_FIEND));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
