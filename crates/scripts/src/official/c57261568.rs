// Flower Cardian Peony with Butterfly
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 57261568;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand, Special Summoned by Tributing Flower Cardian Pine, with more in the Deck to draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (81752019, MZONE),
    (81752019, MZONE),
    (81752019, DECK),
    (81752019, DECK),
    (81752019, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_condition(duel, selfspcon);
    e0.set_target(duel, selfsptg);
    e0.set_operation(duel, selfspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM);
    e2.set_synchro_material_operation(duel, super::c33541430::synop);
    c.register_effect(duel, e2);
}

fn selfspfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FLOWER_CARDIAN]) && !c.is_code(duel, ID)
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, selfspfilter, 1, release_options(c, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(tp, selfspfilter, (1, 1), true, release_options(c, tp))?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST)?;
    Ok(())
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? > 0 {
        let Some(dc) = duel.get_operated_group().get_first() else {
            return Ok(());
        };
        duel.confirm_cards(1 - tp, dc)?;
        if dc.is_set_card(duel, &[SET_FLOWER_CARDIAN]) && dc.is_monster(duel) {
            let deck_ct = duel.get_field_group_count(1 - tp, LOCATION_DECK, 0);
            let ct = deck_ct.min(3);
            if ct == 0 {
                duel.shuffle_hand(tp)?;
                return Ok(());
            }
            duel.break_effect()?;
            let g = duel.get_decktop_group(1 - tp, ct);
            duel.confirm_cards(tp, g)?;
            let mut op = 0;
            if deck_ct > 3 {
                op = duel.select_option(tp, &[aux::stringid(ID, 2), aux::stringid(ID, 3)])?;
            }
            duel.sort_decktop(tp, 1 - tp, ct as u16)?;
            if op == 1 {
                duel.move_to_deck_bottom_count(ct, 1 - tp)?;
            }
        } else {
            duel.break_effect()?;
            duel.send_to_grave(dc, REASON_EFFECT)?;
        }
        duel.shuffle_hand(tp)?;
    }
    Ok(())
}
