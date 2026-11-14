// Gunkan Suship Ikura-class Dreadnought
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::{OperationFn, TargetFn};

const ID: u32 = 75215744;

pub const LISTED_SERIES: &[u16] = &[SET_GUNKAN];

const CARD_SUSHIP_IKURA: u32 = 61027400;
pub const LISTED_NAMES: &[u32] = &[CARD_SUSHIP_SHARI, CARD_SUSHIP_IKURA];

// In the Extra Deck, Xyz Summoned with Gunkan Suship Shari and Ikura, against the opponent's weak Psychic Kappa to deal battle damage through.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_SUSHIP_SHARI, MZONE),
    (CARD_SUSHIP_SHARI, MZONE),
    (CARD_SUSHIP_IKURA, MZONE),
    (CARD_SUSHIP_IKURA, MZONE),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    add_material_effects::<CARD_SUSHIP_IKURA>(duel, c, ID, regtg, regop);
}

// The "gains effects based on material" pair the Gunkan Xyz Monsters share: the first effect's label is bit 1 for Shari, bit 2 for `CODE`.
pub fn add_material_effects<const CODE: u32>(duel: &mut Duel, c: CardId, id: u32, target: TargetFn, operation: OperationFn) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, id);
    e2.set_label(duel, 0);
    e2.set_condition(duel, regcon);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck::<CODE>));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    if a.ep == a.tp {
        return false;
    }
    let Some(rc) = a.eg.and_then(|eg| eg.get_first()) else {
        return false;
    };
    rc.is_controler(duel, a.tp) && rc.is_set_card(duel, &[SET_GUNKAN]) && rc.is_summon_location(duel, LOCATION_EXTRA)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel) && a.e.get_label(duel) > 0
}

pub fn regtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let effs = e.get_label(duel);
    if chk == 0 {
        return Ok((effs & 1 > 0 && duel.is_player_can_draw(tp, 1)) || effs & 2 > 0);
    }
    if effs & 1 > 0 {
        e.set_category(duel, CATEGORY_DRAW);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    } else {
        e.set_category(duel, 0);
    }
    Ok(false)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let effs = e.get_label(duel);
    if effs & 1 > 0 && duel.is_player_can_draw(tp, 1) {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    if effs & 2 > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn valcheck<const CODE: u32>(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let has = |code| g.is_exists(duel, |mc, duel| mc.is_code(duel, code), 1, None);
    let effs = i64::from(has(CARD_SUSHIP_SHARI)) | (i64::from(has(CODE)) << 1);
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, effs);
    }
}
