// Brionac, Dragon of the Ice Barrier
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 50321796;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field with a card in the hand to discard and monsters of the opponent's to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (15025844, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
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
    let rt = duel.get_target_count(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, None);
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
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let ct = e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let eg = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, ct, ct, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, eg, ct, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let rg = tg.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !rg.is_empty() {
        duel.send_to_hand(rg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
