// Gashadokuro, the Skeletal Mayakashi
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 39475024;

// A copy in the Extra Deck, Synchro Summoned with Tsukahagi (a Level 2 Tuner) and Yoko (Level 9), and one in the Graveyard,
// with Yuki-Onna (a Link Monster) and Skull Servant (a Zombie) for the opponent's Raigeki and Dark Hole to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (5325155, MZONE),
    (3486020, MZONE),
    (66870733, MZONE),
    (32274490, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon::<TYPE_LINK, 0>);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

// Shared by the Mayakashi Synchro Monsters: `LEVEL` is the original Level of the Synchro Monster they react to, 0 for any.
pub fn spfilter<const TYPE: u32, const LEVEL: u32>(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_controler(duel, tp)
        && c.get_previous_type_on_field(duel) & TYPE != 0
        && (LEVEL == 0 || c.get_original_level(duel) == LEVEL)
        && (c.is_reason(duel, REASON_BATTLE) || c.is_reason(duel, REASON_EFFECT) && c.get_reason_player(duel) == 1 - tp)
}

pub fn spcon<const TYPE: u32, const LEVEL: u32>(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.eg.is_some_and(|eg| !eg.is_contains(c) && eg.is_exists(duel, |tc, duel| spfilter::<TYPE, LEVEL>(tc, duel, a.tp), 1, None))
}

pub fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_previous_location(duel) == LOCATION_GRAVE
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3100);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

pub fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && c.is_race(duel, RACE_ZOMBIE)
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(rmfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, aux::necro_valley_filter(rmfilter), tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    if !g.is_empty() && duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0 && c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
