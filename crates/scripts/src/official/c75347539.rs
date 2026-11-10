// Valkyrion the Magna Warrior
use super::c25833572::{by_codes, rescon};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 75347539;
const CODES: &[u32] = &[99785935, 39256679, 11549357];

pub const LISTED_NAMES: &[u32] = CODES;

// In the hand and on the field, with the three Magnet Warriors on the field to Tribute and in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (99785935, MZONE),
    (39256679, MZONE),
    (11549357, MZONE),
    (99785935, GRAVE),
    (39256679, GRAVE),
    (11549357, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, super::c25833572::spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let (g, all) = by_codes(duel, duel.get_release_group_full(tp, true, false, REASON_COST), CODES);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && all
        && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| rescon(duel, sg, tp, CODES))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_release_group_full(tp, true, false, REASON_COST).filter(
        duel,
        |c, duel| CODES.iter().any(|&code| c.is_code(duel, code)),
        None,
    );
    let g = aux::select_unselect_group_full(
        duel,
        rg,
        3,
        3,
        &|duel, sg, _, _| rescon(duel, sg, tp, CODES),
        tp,
        HINTMSG_RELEASE,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(ft > 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && CODES
                .iter()
                .all(|&code| duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp, code), tp, LOCATION_GRAVE, 0, 1, None)));
    }
    let mut g = CardSet::default();
    for &code in CODES {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        g.merge(
            duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp, code), tp, LOCATION_GRAVE, 0, 1, 1, None)?
                .unwrap_or_default(),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 3, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if g.len() != 3 || ft < 3 {
        return Ok(());
    }
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
