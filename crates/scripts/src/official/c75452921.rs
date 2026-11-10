// Knightmare Cerberus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 75452921;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property(duel, sg, &|c, duel| c.get_codes_full(duel, Some(lc), sumtype, tp))
    }
}

// In the Extra Deck, Link Summoned from two different monsters, with Raigeki in the hand to discard and Elfnote Lucina in the
// opponent's hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (3573512, MZONE),
    (4035199, MZONE),
    (12580477, HAND),
    (13597785, HAND | OPPONENT),
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
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.get_mutual_linked_group_count(duel) > 0);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_special_summoned(duel) && c.get_sequence(duel) < 5
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && desfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(desfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, desfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    let mutual = e.get_handler(duel).get_mutual_linked_group_count(duel) > 0;
    e.set_label(duel, i64::from(mutual));
    e.set_category(
        duel,
        if mutual {
            CATEGORY_DESTROY + CATEGORY_DRAW
        } else {
            CATEGORY_DESTROY
        },
    );
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? != 0
        && e.get_label(duel) == 1
        && duel.is_player_can_draw(tp, 1)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
