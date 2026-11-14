// Fantastical Dragon Phantazmay
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 78661338;

// In hand for the opponent's Link Summon of Fiendsmith's Requiem with Lacrima the Crimson Tears, and on the field for the
// opponent's Mind Control.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (2463794, EXTRA | OPPONENT),
    (2463794, EXTRA | OPPONENT),
    (28803166, MZONE | OPPONENT),
    (28803166, MZONE | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DRAW + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_link_monster(duel) && !c.is_summon_player(duel, a.tp), 1, None)
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, discon);
    e2.set_cost(duel, discost);
    e2.set_target(duel, stock::negate_destroy_target);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn opponent_links(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group_count(|c, duel| c.is_type(duel, TYPE_LINK), tp, 0, LOCATION_MZONE, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ct = opponent_links(duel, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && ct > 0
            && duel.is_player_can_draw(tp, ct + 1));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32 + 1);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 0, tp, ct as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let ct = opponent_links(duel, tp);
        duel.break_effect()?;
        if duel.draw(p, ct as u32 + 1, REASON_EFFECT)? == ct as i32 + 1 && ct > 0 {
            let g = duel.get_matching_group(CardId::is_able_to_deck, p, LOCATION_HAND, 0, None);
            if g.is_empty() {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, p, HINTMSG_TODECK);
            let sg = g.select(duel, p, ct as u8, ct as u8, None)?;
            duel.break_effect()?;
            duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(re) = a.re else { return false };
    if !(a.rp == 1 - tp && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)) {
        return false;
    }
    let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    g.is_some_and(|g| {
        g.is_exists(
            duel,
            |c, duel| c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE),
            1,
            None,
        )
    }) && duel.is_chain_negatable(a.ev as u8)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
