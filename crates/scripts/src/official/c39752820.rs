// Lib the World Key Blademaster
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 39752820;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_LEGACY];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn spcon(duel: &Duel, _: EffectId, tp: u8) -> bool {
        splimit_link(duel, tp)
    }
}

// In the Extra Deck, Link Summoned with a World Legacy monster in the Graveyard, with World Legacy Struggle in the Deck to Set and Link
// monsters to Link Summon with it as material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13117073, EXTRA),
    (13117073, EXTRA),
    (13179332, MZONE),
    (33112041, MZONE),
    (12143771, MZONE),
    (27918365, GRAVE),
    (93920420, GRAVE),
    (58374719, DECK),
    (58374719, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(|duel, _, _, sp, st, _, _| splimit(duel, sp, st)));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, tdcon);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn splimit_link(duel: &Duel, sp: u8) -> bool {
    duel.is_existing_matching_card(|c, duel| c.is_set_card(duel, &[SET_WORLD_LEGACY]), sp, LOCATION_GRAVE, 0, 1, None)
}

fn splimit(duel: &Duel, sp: u8, st: u32) -> bool {
    st & SUMMON_TYPE_LINK != SUMMON_TYPE_LINK || splimit_link(duel, sp)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WORLD_LEGACY]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.sset(tp, sc)? > 0
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_label_object_group(duel, Some(sc.into()));
        e1.set_condition(duel, |duel, a| {
            a.e.get_label_object_group(duel)
                .and_then(|g| g.get_first())
                .is_some_and(|sc| sc.is_facedown(duel))
                && ctcon(duel, a.e)
        });
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        sc.register_effect(duel, e1);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WORLD_LEGACY]) && c.is_monster(duel)
}

fn ctcon(duel: &Duel, e: EffectId) -> bool {
    !duel.is_existing_matching_card(cfilter, e.get_handler_player(duel), LOCATION_GRAVE, 0, 1, None)
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && a.r == REASON_LINK && c.is_link_summoned(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
