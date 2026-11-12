// Scapeghost
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 67284107;

pub const LISTED_NAMES: &[u32] = &[67284108];

// Three copies on the field to be flipped by the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_ZOMBIE, ATTRIBUTE_DARK)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut ft = if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        1
    } else {
        5
    };
    ft = ft.min(duel.get_location_count(tp, LOCATION_MZONE));
    if ft <= 0 || !duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()) {
        return Ok(());
    }
    let mut i = 0;
    loop {
        if let Some(token) = duel.create_token(tp, ID + 1 + i) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        ft -= 1;
        i = (i + 1) % 4;
        if ft <= 0 || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            break;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
