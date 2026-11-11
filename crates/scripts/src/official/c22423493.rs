// Qliphort Genius
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 22423493;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_MACHINE, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Steel Ogre Grotto #1 (Machine), with the opponent's monster to negate. The search needs
// two monsters Special Summoned at once to a zone it points to, which no card here does.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (29172562, MZONE),
    (29172562, MZONE),
    (29172562, MZONE),
    (29172562, DECK),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_spell_trap_effect(duel)
        || (te.is_active_type(duel, TYPE_LINK) && te.is_activated(duel) && te.get_owner(duel) != e.get_owner(duel))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(
            duel.is_existing_target(CardId::is_negatable, tp, LOCATION_ONFIELD, 0, 1, Some(c.into()))
                && duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELF);
    duel.select_target(tp, CardId::is_negatable, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_OPPO);
    duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let g = duel.get_target_cards(e).unwrap_or_default().filter(
        duel,
        |tc, duel| tc.is_negatable(duel) && tc.is_relate_to_effect(duel, e),
        None,
    );
    if g.len() != 2 {
        return Ok(());
    }
    for tc in g {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(c) && aux::zptgroup(duel, eg, aux::TRUE, c).len() == 2
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_level_above(duel, 5) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
