// Blackfeather Darkrage Dragon
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 60992105;

// Two on the field, with the opponent's attackers (Elfnote Lucina) to deal damage to the player.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DECKDES + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_DAMAGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |_, a| a.ep == a.tp);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 1);
    Ok(false)
}

fn monfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_location(duel, LOCATION_GRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let ct = duel.get_field_group_count(tp, LOCATION_DECK, 0).min(5) as u32;
    let ac = if ct == 1 { 1 } else { duel.announce_number_range(tp, 1, ct)? };
    if duel.discard_deck(tp, ac as u16, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let og = duel.get_operated_group();
    if og.is_exists(duel, monfilter, 1, None) && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 400);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
