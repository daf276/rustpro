// Radiant Typhoon Meghala
use super::c16922142::register_wind_only;
use super::c54143349::selfspcon;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 27755794;

pub const LISTED_NAMES: &[u32] = &[CARD_MYSTICAL_SPACE_TYPHOON];
// On the field with Mystical Space Typhoon (a Quick-Play Spell) in the hand, and Radiant Typhoon monsters in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (CARD_MYSTICAL_SPACE_TYPHOON, HAND), (80538047, DECK), (54143349, DECK)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, selfspcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && (rc.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON) || (rc.is_set_card(duel, &[SET_RADIANT_TYPHOON]) && rc.is_quick_play_spell(duel)))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let code = c.get_code(duel);
    c.is_set_card(duel, &[SET_RADIANT_TYPHOON])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !duel.is_existing_matching_card(
            aux::faceup_filter(move |c, duel| c.is_code(duel, code)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    register_wind_only(duel, e.get_handler(duel), tp, aux::stringid(ID, 2));
    Ok(())
}
