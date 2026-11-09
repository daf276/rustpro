// Branded Opening
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND, MZONE};

const ID: u32 = 36637374;

// Guiding Quem (Despia) in the Deck to add or summon with a card to discard, and in the Graveyard with Albion (Fusion) on the
// field and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (45883110, DECK),
    (45883110, DECK),
    (15025844, HAND),
    (87746184, MZONE),
    (53129443, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, repop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_monster(duel)
        && c.is_set_card(duel, &[SET_DESPIA])
        && (c.is_able_to_hand(duel)
            || (ft > 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp, ft), tp, LOCATION_DECK, 0, 1, None)
                && duel.is_existing_matching_card(
                    |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
                    tp,
                    LOCATION_HAND,
                    0,
                    1,
                    Some(e.get_handler(duel).into()),
                ),
        );
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.discard_hand_full(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD, None)? <= 0 {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp, ft), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.break_effect()?;
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |_, duel| sc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff) && ft > 0,
            |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE).map(drop),
            aux::stringid(ID, 1),
        )?;
    }
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_FUSION)
        });
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_type(duel, TYPE_FUSION)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_EFFECT)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_flag_effect(tp, ID) == 0
            && c.is_able_to_remove(duel)
            && a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        return Ok(true);
    }
    Ok(false)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
