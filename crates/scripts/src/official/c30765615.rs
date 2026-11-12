// Ringowurm, the Dragon Guarding the Hundred Apples
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 30765615;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// Copies in the hand and Graveyard, with Water Spirit and Binding Chain (Normal Monsters, Level 1 and 3) to Synchro Summon Armory Arm
// (Level 4) for the Graveyard effect and to be the Normal Monster of the hand effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (487395, MZONE),
    (487395, MZONE),
    (8058240, MZONE),
    (29071332, EXTRA),
    (29071332, EXTRA),
    (29071332, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(ID + 1),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.get_custom_activity_count(ID, a.tp, ACTIVITY_SPSUMMON) > 0);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
}

fn counterfilter(duel: &Duel, c: CardId) -> bool {
    !(c.is_type(duel, TYPE_SYNCHRO) && c.is_synchro_summoned(duel))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(CardId::is_non_effect_monster),
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        None,
    )
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 100, 100, 2, RACE_WYRM, ATTRIBUTE_LIGHT);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, stats)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(can_summon_token(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if can_summon_token(duel, tp)
        && let Some(token) = duel.create_token(tp, ID + 1)
    {
        if duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)? {
            let e1 = duel.create_effect(a.e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CAN_BE_TUNER);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            token.register_effect(duel, e1);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
