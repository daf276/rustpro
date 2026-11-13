// Cynet Ritual
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 34767865;
const CYNET_TOKEN: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[CYNET_TOKEN];

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_ritual_monster(duel) && c.is_race(duel, RACE_CYBERSE)
    }
}

// Paladin of Storm Dragon to Ritual Summon with Ancient Sorcerer as the material, a second copy in the Graveyard to banish
// for the tokens, and the opponent's Raigeki to empty the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (86401517, HAND),
    (86401517, GRAVE),
    (36821538, MZONE),
    (36821538, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(86401517), Check::SpecialSummonedCode(CYNET_TOKEN)];

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 0, 0, 4, RACE_CYBERSE, ATTRIBUTE_LIGHT)
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater::<Params>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    aux::exccon(duel, a) && duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_monster(duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(cfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, cfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_player_can_special_summon_monster(tp, CYNET_TOKEN, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count(tp, LOCATION_MZONE) < 2
        || !duel.is_player_can_special_summon_monster(tp, CYNET_TOKEN, token_stats())
    {
        return Ok(());
    }
    for _ in 0..2 {
        if let Some(token) = duel.create_token(tp, CYNET_TOKEN) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
