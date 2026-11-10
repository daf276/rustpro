// Machina Fortress
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};
use engine::constants::MSG_CONFIRM_CARDS;

const ID: u32 = 5556499;

// In the hand with Mechanical Snail (a Machine Normal Monster) to discard, and opponent's Gale (targets it) and Cyber Falcon to battle it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (34442949, HAND),
    (34442949, HAND),
    (30655537, MZONE | OPPONENT),
    (2009101, MZONE | OPPONENT),
    (2009101, MZONE | OPPONENT),
];
// The discard when targeted shares its messages with other effects, so it is only reported.
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::Message(MSG_CONFIRM_CARDS)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_operation(duel, hdop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_discardable(duel)
}

// The discardable Machines in the hand, without the card itself when a Necrovalley-like effect or its own state forbids it.
fn discard_group(duel: &Duel, c: CardId) -> CardSet {
    let tp = c.get_controler(duel);
    let mut g = duel.get_matching_group(spfilter, tp, LOCATION_HAND, 0, None);
    if !c.is_able_to_grave_as_cost(duel) {
        g.remove(c);
    }
    if g.is_contains(c) {
        for te in duel.get_player_effect(tp, EFFECT_NECRO_VALLEY).iter() {
            let op = te.get_operation(duel);
            unported!(duel, op.is_some());
            if op.is_none() {
                g.remove(c);
                break;
            }
        }
    }
    g
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    for te in c.get_card_effect(duel, EFFECT_NECRO_VALLEY).iter() {
        let op = te.get_operation(duel);
        unported!(duel, op.is_some());
        if op.is_none() {
            return false;
        }
    }
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && discard_group(duel, c).check_with_sum_greater(duel, CardId::get_level, 8)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, Some(c)) = (a.tp, a.c) else { return Ok(()) };
    let g = discard_group(duel, c);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let sg = g.select_with_sum_greater(duel, tp, CardId::get_level, 8)?;
    duel.send_to_grave(sg, REASON_COST | REASON_DISCARD)?;
    Ok(())
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(()) };
    if a.ep == tp || !re.is_active_type(duel, TYPE_EFFECT) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return Ok(());
    }
    let g = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS);
    if g.is_some_and(|g| g.is_contains(a.e.get_handler(duel))) {
        let hg = duel.get_field_group(tp, 0, LOCATION_HAND);
        if hg.is_empty() {
            return Ok(());
        }
        duel.confirm_cards(tp, hg)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let sg = hg.select(duel, tp, 1, 1, None)?;
        duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)?;
        duel.shuffle_hand(1 - tp)?;
    }
    Ok(())
}
