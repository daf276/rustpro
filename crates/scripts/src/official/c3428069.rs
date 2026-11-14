// Buster Blader, the Destruction Swordmaster
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 3428069;

pub const LISTED_NAMES: &[u32] = &[CARD_BUSTER_BLADER];

// On the field with Raigeki to destroy the opponent's Sangans, which are equipped and sent for the others still there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (12580477, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, HAND | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_BUSTER_BLADER as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
    aux::add_er_equip_limit(duel, c, None, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, 1 - tp)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.is_reason(duel, REASON_EFFECT | REASON_BATTLE)
        && c.is_can_be_effect_target(duel, e)
        && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if let Some(chkc) = chkc {
        return Ok(eg.contains(chkc) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && eg.is_exists(duel, |c, duel| filter(c, duel, e, tp), 1, None));
    }
    let g = eg.filter(duel, |c, duel| filter(c, duel, e, tp), None);
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        g.select(duel, tp, 1, 1, None)?
    } else {
        g.get_first().map(CardSet::from).unwrap_or_default()
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(tc), 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, None)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(|d, duel| desfilter(d, duel, c.get_race(duel)), tp, 0, LOCATION_MZONE, 1, None)
}

fn desfilter(c: CardId, duel: &Duel, rc: u64) -> bool {
    c.is_faceup(duel) && c.is_race(duel, rc)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(e
            .get_handler(duel)
            .get_equip_group(duel)
            .is_exists(duel, |c, duel| tgfilter(c, duel, tp), 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = e
        .get_handler(duel)
        .get_equip_group(duel)
        .filter_select(duel, tp, |c, duel| tgfilter(c, duel, tp), 1, 1, None)?;
    let race = g.get_first().map_or(0, |tc| tc.get_race(duel));
    e.set_label(duel, race as i64);
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destroyable(duel: &Duel, a: &Args) -> CardSet {
    let race = a.e.get_label(duel) as u64;
    duel.get_matching_group(|c, duel| desfilter(c, duel, race), a.tp, 0, LOCATION_MZONE, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = destroyable(duel, a);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = destroyable(duel, a);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
