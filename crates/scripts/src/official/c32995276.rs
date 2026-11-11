// Link Disciple
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 32995276;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_level_below(duel, 4) && c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// Copies in the Extra Deck, and Level 4 or lower Cyberse Normal Monsters (Protron, Digitron) to Link Summon with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (32295838, MZONE),
    (32295838, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    let filter = |c, _: &Duel| lg.is_contains(c);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.break_effect()?;
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    }
    Ok(())
}
