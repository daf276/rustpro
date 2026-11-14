// Eldlich the Mad Golden Lord
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 74889525;

pub const LISTED_SERIES: &[u16] = &[SET_ELDLICH];
pub const MATERIAL_SETCODE: &[u16] = &[SET_ELDLICH];

pub const LISTED_NAMES: &[u32] = &[CARD_GOLDEN_LORD];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_set_card(duel, &[SET_ELDLICH])),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_race_full(duel, RACE_ZOMBIE, Some(fc), sumtype, tp) && c.is_level_above(duel, 5)
        }),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with two Eldlich the Golden Lords (Eldlich Zombies), a copy on the field,
// and the opponent's Sangan to take control of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_GOLDEN_LORD, MZONE),
    (CARD_GOLDEN_LORD, MZONE),
    (CARD_GOLDEN_LORD, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, CARD_GOLDEN_LORD as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_CONTROL);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_cost(duel, ctcost);
    e4.set_target(duel, cttg);
    e4.set_operation(duel, ctop);
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel, e: Option<EffectId>) -> bool {
    c.is_faceup(duel) && c.is_able_to_change_controler(duel) && e.is_none_or(|e| c.is_can_be_effect_target(duel, e))
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let mut ft = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_CONTROL, 0xff);
    if c.is_controler(duel, tp) && c.get_sequence(duel) < 5 {
        ft += 1;
    }
    c.is_race(duel, RACE_ZOMBIE)
        && ft > 0
        && duel.is_existing_target(|tc, duel| filter(tc, duel, None), tp, 0, LOCATION_MZONE, 1, Some(c.into()))
}

fn ctcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel, None));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, None), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, None), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        // Duel.GetControl returns an integer, which is always truthy in Lua.
        duel.get_control(tc, a.tp)?;
        let c = a.e.get_handler(duel);
        let reset = RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3206);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, reset);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_description(duel, 3302);
        e2.set_code(duel, EFFECT_CANNOT_TRIGGER);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
