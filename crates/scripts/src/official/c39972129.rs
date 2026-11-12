// Number 64: Ronin Raccoon Sandayu
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 39972129;
pub const XYZ_NUMBER: u32 = 64;

pub const LISTED_NAMES: &[u32] = &[39972130];

// In the Extra Deck, Xyz Summoned with two Level 2 Beasts, with the opponent's monsters to copy the ATK of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (12482652, MZONE),
    (12482652, MZONE),
    (20129614, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID + 1)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, xyz, sumtype, tp| c.is_race_full(duel, RACE_BEAST, Some(xyz), sumtype, tp));
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, indcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, -2, 0, 1, RACE_BEAST, ATTRIBUTE_EARTH)
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
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()) {
        let Some(token) = duel.create_token(tp, ID + 1) else {
            return Ok(());
        };
        duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        let atk = g.get_max_group(duel, |c, duel| c.get_attack(duel).into()).map_or(0, |(_, atk)| atk);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK);
        e1.set_value(duel, atk as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect(duel, e1);
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_race(duel, RACE_BEAST),
        a.e.get_handler_player(duel),
        LOCATION_MZONE,
        0,
        1,
        Some(c.into()),
    )
}
