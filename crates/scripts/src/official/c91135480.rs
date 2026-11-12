// Time Thief Double Barrel
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 91135480;

// In the Extra Deck, Xyz Summoned with two Level 4 Normal Monsters, with the opponent's Dark Hole to respond to and Sangan to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (9491461, MZONE),
    (9491461, MZONE),
    (23720856, MZONE),
    (23720856, MZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_CONTROL + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let count = |filter: fn(CardId, &Duel) -> bool| sg.filter_count(duel, filter, None);
    (
        count(CardId::is_monster) <= 1 && count(CardId::is_spell) <= 1 && count(CardId::is_trap) <= 1,
        false,
    )
}

fn available_types(duel: &Duel, c: CardId, tp: u8) -> u32 {
    let mut ty = 0;
    if c.is_faceup(duel) {
        ty |= TYPE_MONSTER;
    }
    if duel.is_existing_matching_card(
        aux::faceup_filter(CardId::is_controler_can_be_changed),
        tp,
        0,
        LOCATION_MZONE,
        1,
        None,
    ) {
        ty |= TYPE_SPELL;
    }
    if duel.is_existing_matching_card(CardId::is_negatable_monster, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None) {
        ty |= TYPE_TRAP;
    }
    ty
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let ty = available_types(duel, c, a.tp);
    Ok(chk == 0 && ty > 0 && c.get_overlay_group(duel).is_exists(duel, |c, duel| c.is_type(duel, ty), 1, None))
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = c.get_overlay_group(duel);
    let ty = available_types(duel, c, tp);
    if ty == 0 {
        return Ok(());
    }
    let mg = g.filter(duel, |c, duel| c.is_type(duel, ty), None);
    let sg = aux::select_unselect_group(duel, mg, 1, 3, &rescon, tp, HINTMSG_XMATERIAL)?;
    let card_type = sg.into_iter().fold(0, |card_type, tc| card_type | tc.get_main_card_type(duel));
    duel.send_to_grave(sg, REASON_EFFECT)?;
    duel.raise_single_event(c, EVENT_DETACH_MATERIAL, Some(e), 0, 0, 0, 0)?;
    duel.break_effect()?;
    if card_type & TYPE_MONSTER != 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_value(duel, 400);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    if card_type & TYPE_SPELL != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let tc = duel
            .select_matching_card(
                tp,
                aux::faceup_filter(CardId::is_controler_can_be_changed),
                tp,
                0,
                LOCATION_MZONE,
                1,
                1,
                None,
            )?
            .get_first();
        if let Some(tc) = tc {
            let reason_player = duel.get_reason_player();
            if duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, reason_player)? != 0 {
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_description(duel, 3206);
                e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, EFFECT_CANNOT_ATTACK);
                e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TURN_SET));
                e1.set_value(duel, 1);
                tc.register_effect(duel, e1);
                let e2 = e1.clone_effect(duel);
                e2.set_description(duel, 3302);
                e2.set_code(duel, EFFECT_CANNOT_TRIGGER);
                tc.register_effect(duel, e2);
            }
        }
    }
    if card_type & TYPE_TRAP != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        let g = duel.select_matching_card(tp, CardId::is_negatable_monster, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        if let Some(tc) = g.get_first() {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(e.get_handler(duel));
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_value(duel, RESET_TURN_SET as i32);
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
