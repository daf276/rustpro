// Knightmare Unicorn
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 38342335;

pub const LISTED_SERIES: &[u16] = &[SET_KNIGHTMARE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(99);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property(duel, sg, &|c, duel| c.get_codes_full(duel, Some(lc), sumtype, tp))
    }
}

// In the Extra Deck, Link Summoned from three different monsters, with Raigeki in hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (3573512, MZONE),
    (4035199, MZONE),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_cost(duel, tdcost);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DRAW_COUNT);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 0);
    e2.set_value(duel, Value::Own(drval));
    c.register_effect(duel, e2);
}

fn tdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_deck(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_deck, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, CardId::is_able_to_deck, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    let mutual = e.get_handler(duel).get_mutual_linked_group_count(duel) > 0;
    e.set_label(duel, i64::from(mutual));
    e.set_category(duel, if mutual { CATEGORY_TODECK + CATEGORY_DRAW } else { CATEGORY_TODECK });
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
        && e.get_label(duel) == 1
        && duel.is_player_can_draw(tp, 1)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        if tc.is_location(duel, LOCATION_DECK) && tc.is_controler(duel, tp) {
            duel.shuffle_deck(tp)?;
        }
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn drfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_KNIGHTMARE]) && c.get_mutual_linked_group_count(duel) > 0
}

fn drval(duel: &Duel, e: EffectId) -> i32 {
    let g = duel.get_matching_group(drfilter, e.get_handler_player(duel), LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() {
        return 1;
    }
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32
}
