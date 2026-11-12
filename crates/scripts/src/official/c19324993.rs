// Xtra HERO Infernal Devicer
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, MZONE};

const ID: u32 = 19324993;

// Link Summoned from two HERO monsters, with Elemental HERO Flame Wingman in the Extra Deck and its materials in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (20721928, MZONE),
    (84327329, MZONE),
    (35809262, EXTRA),
    (21844576, DECK),
    (21844576, DECK),
    (58932615, DECK),
    (58932615, DECK),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_HERO], Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, tg);
    e2.set_value(duel, Value::Card(|duel, _, c| c.get_level(duel) as i32 * 100));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| c.is_set_card(duel, &[SET_HERO])),
    );
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_label_object(duel, Some(e));
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_HERO]));
    duel.register_effect(e1, tp);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_facedown(duel)
        && c.is_set_card(duel, &[SET_HERO])
        && !Fusion::material(duel, c.get_original_code(duel)).is_empty()
        && duel.is_existing_matching_card(|tc, duel| thfilter(tc, duel, c), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, fc: CardId) -> bool {
    if c.is_forbidden(duel) || !c.is_able_to_hand(duel) {
        return false;
    }
    Fusion::material(duel, fc.get_original_code(duel))
        .iter()
        .any(|&code| c.is_code(duel, code))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let cg = duel.select_matching_card(tp, |c, duel| filter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(fc) = cg.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, cg)?;
    let tg = duel.get_matching_group(|c, duel| thfilter(c, duel, fc), tp, LOCATION_DECK, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sg = aux::select_unselect_group(duel, tg, 1, 2, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn tg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_race(duel, RACE_FIEND) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
}
