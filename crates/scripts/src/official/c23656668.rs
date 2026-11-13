// Gravity Controller
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 23656668;

// On the field and in the Extra Deck, with Link Spiders on the opponent's field that may land in an Extra Monster Zone to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (98978921, MZONE | OPPONENT),
    (98978921, MZONE | OPPONENT),
    (98978921, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        filter(c, duel)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_condition(duel, lkcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, indescon);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(c.get_sequence(duel) < 5)));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_condition(duel, tdcon);
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    !c.is_type(duel, TYPE_LINK) && c.get_sequence(duel) > 4
}

fn lkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_status(duel, STATUS_SPSUMMON_TURN) && c.is_link_summoned(duel)
}

fn indescon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_sequence(duel) > 4
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    tc.is_some_and(|tc| tc.is_controler(duel, 1 - a.tp) && tc.get_sequence(duel) > 4)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let tc = c.get_battle_target(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel) && tc.is_some_and(|tc| tc.is_able_to_deck(duel)));
    }
    let g = tc.map_or(CardSet::from(c), |tc| CardSet::from_iter([c, tc]));
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), 2, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = c.get_battle_target(duel) else { return Ok(()) };
    if c.is_relate_to_battle(duel) && tc.is_relate_to_battle(duel) {
        duel.send_to_deck(CardSet::from_iter([c, tc]), PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
