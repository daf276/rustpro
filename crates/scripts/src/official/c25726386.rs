// Megalith Aratron
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 25726386;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

// The Ritual Summon from the hand of Aratron and Megalith Bethor: another Megalith, without the discarded card as material.
pub struct TargetParams;

impl RitualParams for TargetParams {
    const FORCEDSELECTION: Option<ForcedSelection> = Some(|duel, e, _, g, _| {
        let c = e.get_handler(duel);
        (!g.is_contains(c), g.is_contains(c))
    });

    fn filter_with_effect(c: CardId, duel: &Duel, e: EffectId) -> bool {
        c.is_set_card(duel, &[SET_MEGALITH]) && c != e.get_handler(duel)
    }
}

pub struct OperationParams;

impl RitualParams for OperationParams {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MEGALITH])
    }
}

// In the hand with Megalith Och to summon by Tributing Kojikocy, and on the field against the opponent's Mind Control, with
// Ritual Monsters in the GY to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (36849933, HAND),
    (1184620, HAND),
    (1184620, MZONE),
    (36849933, GRAVE),
    (30208479, GRAVE),
    (37520316, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, Ritual::target::<TargetParams, RITPROC_GREATER>);
    e1.set_operation(duel, Ritual::operation::<OperationParams, RITPROC_GREATER>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(re) = a.re else { return false };
    if !(re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) && a.rp == 1 - tp && duel.is_chain_negatable(a.ev as u8)) {
        return false;
    }
    let tg = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, |c, duel| c.is_on_field(duel) && c.is_controler(duel, tp), 1, None))
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_monster(duel) && c.is_able_to_deck(duel)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
        }
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let Some(sc) = duel
        .select_matching_card(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    if duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_DECK)
        && duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
