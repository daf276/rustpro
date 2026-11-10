// Tachyon Transmigration
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 8038143;

// Set or in the hand with Galaxy-Eyes monsters on the field (a Tachyon Dragon for the hand activation), and Dark Hole and
// Raigeki on the opponent's side to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (93717133, MZONE),
    (88177324, MZONE),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_TODECK);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_condition(duel, |duel, a| {
        let tp = a.e.get_handler_player(duel);
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GALAXY_EYES_TACHYON_DRAGON])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    });
    c.register_effect(duel, e2);
}

// The chain links of the opponent's monster effects and activations: (chain link, effect).
fn links(duel: &Duel, tp: u8, ev: u32) -> Vec<(u8, EffectId)> {
    (1..=ev as u8)
        .filter_map(|i| {
            let te = duel.get_chain_info::<Option<EffectId>>(i, CHAININFO_TRIGGERING_EFFECT)?;
            let tgp: u8 = duel.get_chain_info(i, CHAININFO_TRIGGERING_PLAYER);
            (tgp != tp && (te.is_monster_effect(duel) || te.is_has_type(duel, EFFECT_TYPE_ACTIVATE))).then_some((i, te))
        })
        .collect()
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GALAXY_EYES])),
        tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    ) && links(duel, tp, a.ev).into_iter().any(|(i, _)| duel.is_chain_negatable(i))
}

fn can_send_to_deck(duel: &Duel, tp: u8, tc: CardId) -> bool {
    tc.is_has_effect(duel, EFFECT_CANNOT_TO_DECK).is_none() && duel.is_player_can_sendto_deck(tp, Some(tc))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let mut ng = CardSet::default();
    let mut dg = CardSet::default();
    for (i, te) in links(duel, tp, a.ev) {
        if duel.is_chain_negatable(i) {
            let tc = te.get_handler(duel);
            ng.insert(tc);
            if tc.is_on_field(duel) && tc.is_relate_to_effect(duel, te) && can_send_to_deck(duel, tp, tc) {
                dg.insert(tc);
            }
        }
    }
    duel.set_target_card(dg);
    duel.set_operation_info(0, CATEGORY_NEGATE, Some(ng), ng.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(dg), dg.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut dg = CardSet::default();
    for (i, te) in links(duel, tp, a.ev) {
        if duel.negate_activation(i) {
            let tc = te.get_handler(duel);
            if tc.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, te) && can_send_to_deck(duel, tp, tc) {
                tc.cancel_to_grave(duel);
                dg.insert(tc);
            }
        }
    }
    duel.send_to_deck(dg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    Ok(())
}
