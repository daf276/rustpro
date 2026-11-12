// Ghost Charon, the Underworld Boatman
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 23379054;

// Copies in the hand to Normal Summon as the only monster, Darkfire Dragon (Level 4 Fusion Monster) in the Graveyard to banish for a
// Level 6 Dragon Synchro Monster (Iron Chain Dragon, Red Rising Dragon), and Elfnote Lucina on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (17881964, GRAVE),
    (17881964, GRAVE),
    (19974580, EXTRA),
    (19974580, EXTRA),
    (66141736, EXTRA),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
}

fn counterfilter(duel: &Duel, c: CardId) -> bool {
    c.is_race(duel, RACE_DRAGON)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0 && duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 1
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.get_race(duel) != RACE_DRAGON);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    Ok(false)
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32, mc: CardSet) -> bool {
    c.is_type(duel, TYPE_FUSION)
        && c.is_able_to_remove(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && duel.is_existing_matching_card(
            |tc, duel| filter2(tc, duel, e, tp, lv + c.get_level(duel), mc),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32, mc: CardSet) -> bool {
    c.get_level(duel) == lv
        && c.is_race(duel, RACE_DRAGON)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, Some(mc), c) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = |tc: CardId, duel: &Duel| filter1(tc, duel, e, tp, c.get_level(duel), c.into());
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && duel.is_existing_target(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel
        .select_target(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let lv = c.get_level(duel) + tc.get_level(duel);
    let g = CardSet::from_iter([c, tc]);
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |sc, duel| filter2(sc, duel, e, tp, lv, g), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
