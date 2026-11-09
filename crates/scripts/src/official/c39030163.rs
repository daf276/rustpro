// Galaxy-Eyes Full Armor Photon Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 39030163;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

// In the Extra Deck, Xyz Summoned on Number 62 or with three Level 8 Normal Monsters, with Sangan on the opponent's side to destroy. No
// Equip Card scenario equips it, so the attach effect is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (31801517, MZONE),
    (10949074, MZONE),
    (32626733, MZONE),
    (13140300, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, hint_selected_effect);
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, hint_selected_effect])
    });
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let sumtype = u64::from(SUMMON_TYPE_XYZ);
    c.is_set_card_full(duel, &[SET_GALAXY_EYES], Some(xyzc), sumtype, tp)
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), sumtype, tp)
        && !c.is_summon_code(duel, Some(xyzc), sumtype, tp, &[ID])
        && c.is_faceup(duel)
}

// Cost.HintSelectedEffect.
fn hint_selected_effect(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_OPSELECTED, 1 - a.tp, a.e.get_description(duel));
    Ok(false)
}

fn attachfilter(c: CardId, duel: &Duel, xyzc: CardId, e: EffectId, tp: u8) -> bool {
    c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT) && c.is_can_be_effect_target(duel, e)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let eqg = c.get_equip_group(duel);
    let filter = |mc: CardId, duel: &Duel| attachfilter(mc, duel, c, e, tp);
    if let Some(chkc) = chkc {
        return Ok(eqg.is_contains(chkc) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(eqg.is_exists(duel, filter, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tg = eqg.filter_select(duel, tp, filter, 1, 2, None)?;
    duel.set_target_card(tg);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e) = (a.e.get_handler(duel), a.e);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let tg = duel
        .get_target_cards(e)
        .unwrap_or_default()
        .filter(duel, |tc, duel| !tc.is_immune_to_effect(duel, e), None);
    if !tg.is_empty() {
        duel.overlay(c, tg)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
