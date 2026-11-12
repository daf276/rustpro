// Number C92: Heart-eartH Chaos Dragon
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 47017574;
pub const XYZ_NUMBER: u32 = 92;
const HEART_EARTH_DRAGON: u32 = 97403510;

pub const LISTED_NAMES: &[u32] = &[HEART_EARTH_DRAGON];

// On the field beside Black Luster Soldier and Elfnote Lucina, which attack the opponent's Sangan for battle damage; the negate effect needs
// Number 92: Heart-eartH Dragon attached, which only Rank-Up-Magic does.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (10000100, MZONE),
    (13597785, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 4;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, reccon);
    e2.set_target(duel, rectg);
    e2.set_operation(duel, recop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(
        duel,
        EFFECT_FLAG_CANNOT_INACTIVATE + EFFECT_FLAG_CANNOT_NEGATE + EFFECT_FLAG_CANNOT_DISABLE,
    );
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, discon);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn reccon(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && a.eg.and_then(|eg| eg.get_first()).is_some_and(|tc| tc.is_controler(duel, a.tp))
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(a.ev);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, a.ev as i32);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_code(duel, HEART_EARTH_DRAGON), 1, None)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_negatable, a.tp, 0, LOCATION_ONFIELD, 1, None))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_negatable, a.tp, 0, LOCATION_ONFIELD, None);
    for tc in g {
        let codes: &[u32] = if tc.is_type(duel, TYPE_TRAPMONSTER) {
            &[EFFECT_DISABLE, EFFECT_DISABLE_EFFECT, EFFECT_DISABLE_TRAPMONSTER]
        } else {
            &[EFFECT_DISABLE, EFFECT_DISABLE_EFFECT]
        };
        for &code in codes {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}
