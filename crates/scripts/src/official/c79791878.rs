// Shining Sarcophagus
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT, SZONE};

const ID: u32 = 79791878;
const DARK_MAGICIAN_GIRL_THE_MAGICIANS_APPRENTICE: u32 = 2501624;
const MONSTER_REBORN: u32 = 83764718;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with the Apprentice in the Deck to search, and the opponent's Monster Reborn reviving Sangan against
// Spells in hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (DARK_MAGICIAN_GIRL_THE_MAGICIANS_APPRENTICE, DECK),
    (DARK_MAGICIAN_GIRL_THE_MAGICIANS_APPRENTICE, DECK),
    (12580477, HAND),
    (12580477, HAND),
    (MONSTER_REBORN, HAND | OPPONENT),
    (MONSTER_REBORN, HAND | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_CUSTOM + ID);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, |duel, _| duel.get_current_phase() != PHASE_DAMAGE);
    e4.set_cost(duel, tgcost);
    e4.set_target(duel, tgtg);
    e4.set_operation(duel, tgop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_operation(duel, regop);
    c.register_effect(duel, e5);
}

// The group e4 and e5 share as their label object.
fn group_key(duel: &Duel, e: EffectId) -> u64 {
    u64::from(ID) << 16 | e.get_owner(duel).index() as u64
}

fn label_group(duel: &Duel, e: EffectId) -> CardSet {
    duel.get_script_group(group_key(duel, e)).unwrap_or_default()
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, ID) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, p: u8) -> bool {
    c.is_summon_player(duel, p)
        && c.is_summon_location(duel, LOCATION_GRAVE)
        && c.is_able_to_grave(duel)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_can_be_effect_target(duel, e)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| tgfilter(c, duel, e, 1 - tp), None);
    if !tg.is_empty() {
        for tc in tg {
            tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
        }
        let g = if duel.get_current_chain() == 0 {
            CardSet::default()
        } else {
            label_group(duel, e)
        };
        let mut g = g.filter(duel, |c, duel| c.get_flag_effect(duel, ID) != 0, None);
        g.merge(tg);
        duel.set_script_group(group_key(duel, e), Some(g));
        duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_discardable(duel)
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = label_group(duel, e).filter(duel, |c, duel| tgfilter(c, duel, e, 1 - tp), None);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && tgfilter(chkc, duel, e, 1 - tp));
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let mut tg = g;
    if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        tg = g.select(duel, tp, 1, 1, None)?;
    }
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(tg), 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}
