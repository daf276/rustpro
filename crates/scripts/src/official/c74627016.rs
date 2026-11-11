// T.G. Tank Grub
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 74627016;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// On the field with T.G. Screw Serpent (Level 4 Tuner), Synchro Summoning T.G. Wonder Magician (Level 5) with it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (11234702, MZONE), (98558751, EXTRA), (98558751, EXTRA)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_NONTUNER);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, sc| sc.is_set_card(duel, &[SET_TG]).into()));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, tkcon);
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop);
    c.register_effect(duel, e2);
}

fn tkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && a.r == REASON_SYNCHRO
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_set_card(duel, &[SET_TG]))
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_TG, TYPES_TOKEN, 0, 0, 1, RACE_MACHINE, ATTRIBUTE_EARTH)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_monster_full(tp, ID + 1, token_stats(), POS_FACEUP_ATTACK, tp, 0));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if duel.is_player_can_special_summon_monster_full(tp, ID + 1, token_stats(), POS_FACEUP_ATTACK, tp, 0)
        && let Some(token) = duel.create_token(tp, ID + 1)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}
