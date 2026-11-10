// Galatea, the Orcust Automaton
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, REMOVED};

const ID: u32 = 30741503;

pub struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_ORCUST], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// In the Extra Deck for its Link Summon from Girsu, on the field, and with banished Machines to shuffle.
// No Orcust Spell/Trap is ported, so nothing is Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (69811710, MZONE),
    (69811710, MZONE),
    (69811710, REMOVED),
    (69811710, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_linked(duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| !babelcon(duel, a));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_condition(duel, babelcon);
    c.register_effect(duel, e3);
}

fn babelcon(duel: &Duel, a: &Args) -> bool {
    duel.is_player_affected_by_effect(a.tp, CARD_ORCUSTRATED_BABEL).is_some()
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ORCUST]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    let og = duel.get_operated_group();
    if og.is_empty() {
        return Ok(());
    }
    if og.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.sset(tp, sg)?;
    }
    Ok(())
}
