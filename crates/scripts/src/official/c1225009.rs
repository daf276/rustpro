// Arianna the Labrynth Servant
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 1225009;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with Big Welcome Labrynth Set to return it to the hand, Arias in the Deck to search and in the hand to summon,
// and Trap Hole in the hand to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (92714517, SZONE),
    (92714517, SZONE),
    (73602965, DECK),
    (73602965, DECK),
    (73602965, HAND),
    (4206964, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DRAW + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, drcon);
    e3.set_target(duel, stock::draw_target::<1>);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LABRYNTH]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_EFFECT)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| {
        a.rp == a.tp
            && re.is_trap_effect(duel)
            && re.get_handler(duel).get_original_type(duel) == TYPE_TRAP
            && a.eg.unwrap_or_default().is_exists(duel, cfilter, 1, None)
    })
}

fn ffilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn stfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let ffilter = |c: CardId, duel: &Duel| ffilter(c, duel, e, tp);
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(ffilter, tp, LOCATION_HAND, 0, 1, None);
    let b2 = duel.is_existing_matching_card(stfilter, tp, LOCATION_HAND, 0, 1, None);
    if (b1 || b2) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        match duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])? {
            Some(1) => {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let g = duel.select_matching_card(tp, ffilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
                if !g.is_empty() {
                    duel.break_effect()?;
                    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
                }
            }
            Some(2) => {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
                let g = duel.select_matching_card(tp, stfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
                if !g.is_empty() {
                    duel.break_effect()?;
                    duel.sset_full(tp, g, tp, false)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}
