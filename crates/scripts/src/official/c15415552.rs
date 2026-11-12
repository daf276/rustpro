// Mortilux Heruvur
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 15415552;

// In the Extra Deck, Xyz Summoned with four Level 8 Normal Monsters (Black Luster Soldier), with Raigeki in hand to send the
// opponent's monsters to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (10000100, MZONE),
    (10000100, MZONE),
    (10000100, MZONE),
    (12580477, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1a.set_code(duel, EVENT_CUSTOM + ID);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, attachtg);
    e1a.set_operation(duel, attachop);
    e1a.set_label_object_group(duel, Some(CardSet::default()));
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1b.set_code(duel, EVENT_TO_GRAVE);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_label_object(duel, Some(e1a));
    e1b.set_operation(duel, regop);
    c.register_effect(duel, e1b);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE);
    e2a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_condition(duel, xyzmatcountcon::<2>);
    e2a.set_value(duel, 1);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_GRAVE, LOCATION_GRAVE);
    e3.set_condition(duel, xyzmatcountcon::<3>);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, xyzmatcountcon::<4>);
    e4.set_cost(duel, Cost::detach_from_self::<3, 3>);
    e4.set_target(duel, sendtogytg);
    e4.set_operation(duel, sendtogyop);
    c.register_effect(duel, e4);
}

fn attachfilter(c: CardId, duel: &Duel, tp: u8, e: EffectId, xyzc: CardId) -> bool {
    c.is_location(duel, LOCATION_GRAVE)
        && c.is_monster(duel)
        && c.is_controler(duel, 1 - tp)
        && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
        && c.is_can_be_effect_target(duel, e)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_damage_step() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let tg =
        a.eg.unwrap_or_default()
            .filter(duel, |tc, duel| attachfilter(tc, duel, tp, e, c), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e1a) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e1a.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e1a.set_label_object_group(duel, Some(g));
    duel.raise_single_event(c, EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |tc, duel| attachfilter(tc, duel, tp, e, c), None);
    if chk == 0 {
        return Ok(c.is_xyz_monster(duel) && !g.is_empty());
    }
    let tc = if g.len() == 1 {
        g
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
        g.select(duel, tp, 1, 1, None)?
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(tc), 1, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, tc)?;
    }
    Ok(())
}

fn xyzmatcountcon<const REQUIRED: usize>(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_overlay_count(duel) >= REQUIRED
}

fn sendtogytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), 1, tp, 0);
    Ok(false)
}

fn sendtogyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
