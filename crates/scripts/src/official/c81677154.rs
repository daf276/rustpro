// Mementotlan Tatsunootoshigo
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 81677154;

// In the hand and on the field, with Mementotlan Angwitch (Level 3) in the Deck to send to the GY.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (54550967, DECK), (54550967, DECK), (54550967, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, 0, None);
    g.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_MEMENTO]), None) == g.len()
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_monster(duel) && c.is_level_below(duel, lv) && c.is_able_to_grave(duel)
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO])
        && c.is_faceup(duel)
        && c.has_level(duel)
        && duel.is_existing_matching_card(
            |g, duel| tgfilter(g, duel, c.get_original_level(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, tp), tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let tc = duel
        .select_matching_card(tp, |c, duel| desfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.destroy(tc, REASON_EFFECT)? > 0
    {
        let lv = tc.get_original_level(duel);
        let g = duel.get_matching_group(|c, duel| tgfilter(c, duel, lv), tp, LOCATION_DECK, 0, None);
        let rescon = |duel: &Duel, sg: CardSet, _, _| {
            let classes = sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
            (
                sg.get_sum(duel, |c, duel| i64::from(c.get_level(duel))) <= i64::from(lv) && classes == sg.len(),
                classes != sg.len(),
            )
        };
        let sg = aux::select_unselect_group_full(duel, g, 1, lv as usize, &rescon, tp, HINTMSG_TOGRAVE, Some(&rescon), false)?;
        if !sg.is_empty() {
            duel.send_to_grave(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
