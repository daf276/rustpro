// Morphing Jar #2
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 79106360;

// Face-down on the field for both players, to be flipped by an attack, with monsters of both to send to the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE | OPPONENT),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK + CATEGORY_TOGRAVE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_ALL, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_ALL, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_ALL, LOCATION_DECK as i32);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    !c.is_status(duel, STATUS_BATTLE_DESTROYED) && c.is_able_to_deck(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.send_to_deck(rg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    let rg = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK) && c.is_monster(duel), None);
    let ct1 = rg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
    let ct2 = rg.len() - ct1;
    if ct1 > 0 {
        duel.shuffle_deck(tp)?;
    }
    if ct2 > 0 {
        duel.shuffle_deck(1 - tp)?;
    }
    duel.break_effect()?;
    let g1 = if ct1 > 0 { sp(duel, e, tp, ct1)? } else { None };
    let g2 = if ct2 > 0 { sp(duel, e, 1 - tp, ct2)? } else { None };
    duel.special_summon_complete()?;
    for g in [g1, g2].into_iter().flatten() {
        duel.shuffle_set_card(g)?;
    }
    Ok(())
}

fn sp(duel: &mut Duel, e: EffectId, tp: u8, ct: usize) -> Script<Option<CardSet>> {
    let g = duel.get_field_group(tp, LOCATION_DECK, 0);
    let dt = g.len();
    if dt == 0 {
        return Ok(None);
    }
    let mut monsters: Vec<CardId> = g.into_iter().filter(|c| c.is_monster(duel)).collect();
    monsters.sort_by_key(|c| std::cmp::Reverse(c.get_sequence(duel)));
    monsters.truncate(ct);
    let mut g: CardSet = monsters.iter().copied().collect();
    let last = monsters.last().copied();
    // The script indexes the nil `last`, a Lua error that ends the operation.
    unported!(duel, last.is_none());
    let conf = last.map_or(0, |last| dt - last.get_sequence(duel) as usize);
    duel.confirm_decktop(tp, conf)?;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    g.keep_matching(duel, |c, duel| spfilter(c, duel, e, tp), None);
    if g.len() as i32 > ft {
        unported!(duel, ft < 0);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        g = g.select(duel, tp, ft as u8, ft as u8, None)?;
    }
    for tc in g {
        duel.disable_shuffle_check(true);
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
    }
    let rest = conf as i32 - g.len() as i32;
    if rest > 0 {
        duel.discard_deck(tp, rest as u16, REASON_EFFECT | REASON_EXCAVATE)?;
    }
    Ok(Some(g))
}
