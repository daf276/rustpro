// Transcicada
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 1799464;

pub const LISTED_NAMES: &[u32] = &[ID + 1];
// In the Graveyard, with Monster Reborn to Special Summon it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (83764718, HAND), (83764718, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tktg);
    e1.set_operation(duel, tkop);
    c.register_effect(duel, e1);
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster(tp, ID + 1, MonsterStats::new(0, TYPES_TOKEN, 0, 0, 3, RACE_INSECT, ATTRIBUTE_EARTH))
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
    let (e, tp) = (a.e, a.tp);
    if can_summon_token(duel, tp) {
        let Some(token) = duel.create_token(tp, ID + 1) else {
            return Ok(());
        };
        duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_absolute_range(duel, tp, 1, 0);
        e1.set_target_card(duel, |duel, _, c| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_INSECT)
        });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect_full(duel, e1, true);
        let e2 = aux::create_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| !c.is_original_race(duel, RACE_INSECT));
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect_full(duel, e2, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
