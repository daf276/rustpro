// Dewloren, Tiger King of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 70583986;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(sc), sumtype, tp)
        })
    }
}

// In the Extra Deck, Synchro Summoned with Defender of the Ice Barrier (a Level 3 Tuner) and Water Element (a Level 3 WATER), or on the field with more
// monsters to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (82498947, MZONE),
    (82498947, MZONE),
    (3732747, MZONE),
    (3732747, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let handler = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && filter(chkc, duel) && chkc != handler);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, 0, 1, Some(handler.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, 0, 1, 12, Some(handler.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let rg = duel.get_target_cards(e).unwrap_or_default();
    duel.send_to_hand(rg, PLAYER_NONE, REASON_EFFECT)?;
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        duel.break_effect()?;
        let og = duel.get_operated_group();
        let ct = og.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
        if ct > 0 {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
            e1.set_value(duel, ct as i32 * 500);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
