// Magical Musketeer Max
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 71791814;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_level_below(duel, 8) && c.is_set_card_full(duel, &[SET_MAGICAL_MUSKET], Some(lc), sumtype, tp)
    }
}

// Link Summoned from Magical Musketeer Kidbrave, with Magical Musket - Desperado and another Kidbrave in the Deck, and a monster of the opponent.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (5230799, MZONE),
    (5230799, MZONE),
    (5230799, DECK),
    (20745268, DECK),
    (20745268, DECK),
    (24154052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_HAND, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGICAL_MUSKET]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MAGICAL_MUSKET]) && c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let monct = duel.get_field_group_count(tp, 0, LOCATION_MZONE);
    let stct = duel.get_matching_group_count(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, None);
    let b1 = monct > 0 && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = stct > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    match duel.select_effect(tp, &[(b1, aux::stringid(ID, 0)), (b2, aux::stringid(ID, 1))])? {
        Some(1) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            e.set_operation(duel, thop);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            e.set_operation(duel, spop);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_field_group_count(tp, 0, LOCATION_MZONE);
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if ct == 0 || g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 1, ct, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, tp, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = duel.get_matching_group_count(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, None);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if ct == 0 || g.is_empty() || ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let sg = aux::select_unselect_group(duel, g, 1, ct.min(ft as usize), &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
