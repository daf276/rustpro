// Obedience Schooled
use super::c14878871::{descon, desop};
use crate::prelude::*;
use crate::testing::{DECK, HAND, OPPONENT};

const ID: u32 = 72537897;

// Copies in hand, four Level 1-2 Beast Effect Monsters in the Deck (Key Mouse, Nimble Momonga, Little Chimera, Comic Cat),
// and the opponent's Raigeki to empty the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (135598, DECK),
    (22567609, DECK),
    (68658728, DECK),
    (72921536, DECK),
    (12580477, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_BEAST)
        && c.is_level_below(duel, 2)
        && c.is_type(duel, TYPE_EFFECT)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn summonable(duel: &Duel, tp: u8, g: CardSet) -> bool {
    duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.get_location_count(tp, LOCATION_MZONE) > 2
        && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(summonable(duel, tp, g));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 3, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if summonable(duel, tp, g) {
        let sg = aux::select_unselect_group(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
        let fid = c.get_field_id(duel);
        for tc in sg {
            duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e2);
        }
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e3.set_code(duel, EVENT_PHASE + PHASE_END);
        e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e3.set_count_limit(duel, 1, 0);
        e3.set_label(duel, fid.into());
        e3.set_label_object_group(duel, Some(sg));
        e3.set_condition_mut(duel, descon::<ID>);
        e3.set_operation(duel, desop::<ID>);
        duel.register_effect(e3, tp);
        duel.special_summon_complete()?;
    }
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_target_range(duel, 1, 0);
        e1.set_target_card(duel, |duel, _, c| c.get_race(duel) != RACE_BEAST);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}
