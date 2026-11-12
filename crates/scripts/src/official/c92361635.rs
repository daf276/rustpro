// Atomic Scrap Dragon
use super::c76774528::add_spsummon_effect;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 92361635;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_SCRAP], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field with a card to destroy and cards in the opponent's GY to return to the Deck, and in the Extra Deck, destroyed by the
// opponent's Raigeki with Scrap monsters in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (83135907, GRAVE),
    (4334811, GRAVE),
    (5318639, GRAVE | OPPONENT),
    (5318639, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    add_spsummon_effect(duel, c, ID);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(CardId::is_able_to_deck, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g1, 1, 0, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g2 = duel.select_target(tp, CardId::is_able_to_deck, tp, 0, LOCATION_GRAVE, 1, 3, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g2, g2.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let op_cards = |duel: &Duel, category| {
        duel.get_operation_info(0, category)
            .and_then(|info| info.op_cards)
            .unwrap_or_default()
    };
    let (g1, g2) = (op_cards(duel, CATEGORY_DESTROY), op_cards(duel, CATEGORY_TODECK));
    if g1.get_first().is_some_and(|tc| tc.is_relate_to_effect(duel, e)) && duel.destroy(g1, REASON_EFFECT)? != 0 {
        let hg = g2.filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
        duel.send_to_deck(hg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
