// Divine Temple of the Snake-Eye
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

use super::c90241276::treat_as_continuous_spell;

const ID: u32 = 53639887;

pub const LISTED_SERIES: &[u16] = &[SET_SNAKE_EYE];

const SNAKE_EYE_ASH: u32 = 9674034;
const SNAKE_EYES_POPLAR: u32 = 90241276;

// In the hand to activate, with Snake-Eye Ash and Snake-Eyes Poplar in the Deck and GY to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (SNAKE_EYE_ASH, DECK),
    (SNAKE_EYE_ASH, GRAVE),
    (SNAKE_EYES_POPLAR, DECK),
    (SNAKE_EYES_POPLAR, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_level(duel, &[1]) && c.is_attribute(duel, ATTRIBUTE_FIRE));
    e2.set_value(duel, 1100);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e3.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None))
    });
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    let p = c.get_owner(duel);
    c.is_set_card(duel, &[SET_SNAKE_EYE])
        && c.is_monster(duel)
        && duel.get_location_count(p, LOCATION_SZONE) > 0
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, p)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(
        aux::necro_valley_filter(cfilter),
        tp,
        LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
        0,
        None,
    );
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        if let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first()
            && duel.move_to_field(tc, tp, tc.get_owner(duel), LOCATION_SZONE, POS_FACEUP, true)?
        {
            treat_as_continuous_spell(duel, e.get_handler(duel), tc);
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_original_type(duel, TYPE_MONSTER)
        && c.is_continuous_spell(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
