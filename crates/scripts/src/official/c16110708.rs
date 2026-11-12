// Number S38: Titanic Galaxy
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 16110708;
const NUMBER_38_TITANIC_GALAXY: u32 = 63767246;
pub const XYZ_NUMBER: u32 = 38;

pub const LISTED_NAMES: &[u32] = &[NUMBER_38_TITANIC_GALAXY];

// In the Extra Deck, Xyz Summoned over Number 38: Hope Harbinger Dragon Titanic Galaxy, with the opponent's Spell Cards to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (NUMBER_38_TITANIC_GALAXY, MZONE),
    (NUMBER_38_TITANIC_GALAXY, MZONE),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(9);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 200));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, attachtg);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_base_attack(duel, 1500));
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[NUMBER_38_TITANIC_GALAXY]) && c.is_faceup(duel)
}

fn attachfilter(c: CardId, duel: &Duel, xc: CardId, tp: u8) -> bool {
    c.is_spell_trap(duel) && c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && attachfilter(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| attachfilter(tc, duel, c, tp), tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_target(tp, |tc, duel| attachfilter(tc, duel, c, tp), tp, 0, LOCATION_ONFIELD, 1, 2, None)?;
    let mut targets = g.unwrap_or_default().into_iter();
    // The two targets' indices plus one, packed in the capture, where 0 is no card.
    let packed = |tc: Option<CardId>| tc.map_or(0, |tc| tc.index() as i64 + 1);
    let capture = packed(targets.next()) | packed(targets.next()) << 32;
    duel.set_chain_limit(
        |duel, e, _, _, capture| {
            let rc = e.get_handler(duel).index() as i64 + 1;
            rc != capture & 0xffff_ffff && rc != capture >> 32
        },
        capture,
    );
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let tg = duel.get_target_cards(e).unwrap_or_default().filter(
        duel,
        |tc, duel| attachfilter(tc, duel, c, tp) && !tc.is_immune_to_effect(duel, e),
        None,
    );
    if !tg.is_empty() {
        duel.overlay_full(c, tg, true)?;
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
        e1.set_value(duel, 1500);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    let e2 = duel.create_effect(c);
    e2.set_description(duel, 3205);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e2);
    Ok(())
}
