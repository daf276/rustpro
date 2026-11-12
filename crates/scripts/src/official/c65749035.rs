// Gungnir, Dragon of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 65749035;

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

// In the Extra Deck, Synchro Summoned with Defender of the Ice Barrier (a Level 3 Tuner) and Hyosube (a Level 4 WATER), with cards in the hand to discard and
// monsters of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (82498947, MZONE),
    (82498947, MZONE),
    (2118022, MZONE),
    (2118022, MZONE),
    (15025844, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rt = duel.get_target_count(aux::TRUE, tp, 0, LOCATION_ONFIELD, None).min(2);
    if chk == 0 {
        return Ok(aux::ice_barrier_discard_cost_check(duel, tp, costfilter, true, 1));
    }
    let cg = aux::ice_barrier_discard_cost(duel, tp, costfilter, true, (1, rt))?;
    e.set_label(duel, i64::from(cg));
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let ct = e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let eg = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, ct, ct, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, eg, ct, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let rg = duel.get_target_cards(a.e).unwrap_or_default();
    if !rg.is_empty() {
        duel.destroy(rg, REASON_EFFECT)?;
    }
    Ok(())
}
