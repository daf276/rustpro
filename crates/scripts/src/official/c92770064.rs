// Sunavalon Dryatrentiay
use engine::constants::MSG_CONFIRM_CARDS;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 92770064;

pub const LISTED_SERIES: &[u16] = &[SET_SUNAVALON];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// On the field and in the Extra Deck, with Link Monsters (Sunavalon Dryas, Sunvine Healer) to be linked to and released, and monsters
// of the opponent's to destroy. The search needs an unported Sunavalon Spell/Trap.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (65563871, MZONE),
    (65563871, MZONE),
    (93896655, MZONE),
    (93896655, MZONE),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Report(&Check::Message(MSG_CONFIRM_CARDS))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IGNORE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| rp != e.get_handler_player(duel)));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, |duel, a, _| {
        a.e.set_label(duel, 100);
        Ok(true)
    });
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUNAVALON]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn costfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_faceup(duel) && c.is_link_monster(duel) && lg.is_contains(c)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lg = e.get_handler(duel).get_linked_group(duel);
    let dg = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    let check = aux::release_check_target(dg);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(
            !dg.is_empty() && duel.check_release_group_cost(tp, |c, duel| costfilter(c, duel, lg), (1, 1), false, Some(&check), None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let sg = duel.select_release_group_cost(tp, |c, duel| costfilter(c, duel, lg), (1, 1), false, Some(&check), None)?;
    duel.release(sg, REASON_COST)?;
    let lk = sg.get_first().map_or(0, |c| c.get_link(duel));
    e.set_label(duel, i64::from(lk));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), lk as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, a.e.get_label(duel) as u8, None)?;
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
