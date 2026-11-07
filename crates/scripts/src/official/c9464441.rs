// Adamancipator Risen - Dragite
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 9464441;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field with a copy in the GY, WATER like the negation needs, and the opponent's Raigeki in hand to negate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (12580477, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 4);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 0, 1 - a.tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) < 5 {
        return Ok(());
    }
    duel.confirm_decktop(tp, 5)?;
    let hg = duel.get_matching_group(|c, duel| c.is_able_to_hand(duel), tp, 0, LOCATION_ONFIELD, None);
    let g = duel.get_decktop_group(tp, 5);
    let ct = hg.len().min(g.filter_count(duel, |c, duel| c.is_race(duel, RACE_ROCK), None));
    if ct > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let sg = hg.select(duel, tp, 1, ct as u8, None)?;
        duel.hint_selection(sg);
        if duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? != 0 {
            duel.shuffle_hand(1 - tp)?;
        }
        duel.disable_shuffle_check(true);
    }
    duel.move_to_deck_bottom_count(5, tp)?;
    duel.sort_deckbottom(tp, tp, 5)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.ep == 1 - a.tp
        && re.is_spell_trap_effect(duel)
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(|c, duel| c.is_attribute(duel, ATTRIBUTE_WATER), a.tp, LOCATION_GRAVE, 0, 1, None)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re)) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
