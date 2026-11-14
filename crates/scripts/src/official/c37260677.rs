// Zoroa, the Magistus Verethragna
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 37260677;

pub const LISTED_SERIES: &[u16] = &[SET_MAGISTUS];
pub const MATERIAL_SETCODE: &[u16] = &[SET_MAGISTUS];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_MAGISTUS], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_type_full(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK, Some(fc), sumtype, tp)
        }),
    ];
}

// On the field with Magistus Chorozo for the cost, and the opponent's Elfnote Lucina to equip and to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (66532962, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
    });
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_EFFECT) && c.is_faceup(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(eqfilter(chkc, duel, tp)
            && ((chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp))
                || (chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp))));
    }
    let filter = |c, duel: &Duel| eqfilter(c, duel, tp);
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, LOCATION_MZONE, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, tp, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_type(duel, TYPE_EFFECT) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && duel.get_location_count(tp, LOCATION_SZONE) > 0 {
        c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, None)?;
    } else if tc.is_controler(duel, 1 - tp) {
        duel.send_to_grave(tc, REASON_RULE)?;
    }
    Ok(())
}

fn discostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGISTUS]) && c.is_monster_card(duel) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, discostfilter, LOCATION_ONFIELD)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.negate_effect(a.ev as u8) {
        let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.hint_selection(sg);
            duel.break_effect()?;
            duel.destroy(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
