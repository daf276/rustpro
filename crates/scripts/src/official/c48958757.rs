// Gladiator Beast Claudius
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 48958757;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR_BEAST];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(gladiator_beast); 5];
}

fn gladiator_beast(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_GLADIATOR_BEAST], Some(fc), sumtype, tp)
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(|duel, e, _, _, _, _, _| {
        !e.get_handler(duel).is_location(duel, LOCATION_EXTRA)
    }));
    const SUMTYPE: Option<u32> = Some(1);

    fn group(duel: &Duel, tp: u8) -> CardSet {
        let filter = |c: CardId, duel: &Duel| c.is_monster(duel) && c.is_able_to_deck_or_extra_as_cost(duel);
        duel.get_matching_group(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, tp: u8, _: CardId) -> Script<()> {
        let fdg = g.filter(duel, CardId::is_facedown, None);
        let gyg = g.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
        if !fdg.is_empty() {
            duel.confirm_cards(1 - tp, fdg)?;
        }
        if !gyg.is_empty() {
            duel.hint_selection(gyg);
        }
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

// Contact Summoned from Gladiator Beast Hoplomus and Andal on the field and in the GY, with Hoplomus in the Deck to summon and
// Heraklinos in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (4253484, MZONE),
    (4253484, MZONE),
    (90582719, MZONE),
    (90582719, GRAVE),
    (90582719, GRAVE),
    (4253484, GRAVE),
    (4253484, DECK),
    (4253484, DECK),
    (27346636, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
            && duel.is_player_affected_by_effect(a.tp, EFFECT_BP_TWICE).is_none()
    });
    e1.set_operation(duel, doublebattlephase);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel))
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn doublebattlephase(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, EFFECT_BP_TWICE).is_some() {
        return Ok(());
    }
    let turn_ct = duel.get_turn_count();
    let ct = if duel.is_turn_player(tp) && duel.is_battle_phase() { 2 } else { 1 };
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_BP_TWICE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, 1);
    e1.set_labels(duel, &[ct, turn_ct.into()]);
    e1.set_condition(duel, |duel, a| {
        let &[ct, turn_ct] = a.e.get_labels(duel) else { return false };
        ct == 1 || i64::from(duel.get_turn_count()) != turn_ct
    });
    e1.set_reset_full(duel, RESET_PHASE | PHASE_BATTLE | RESET_SELF_TURN, ct as u16);
    duel.register_effect(e1, tp);
    Ok(())
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && c.is_can_be_special_summoned(duel, e, 100, tp, false, false)
}

fn extraspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 11)
        && c.is_set_card(duel, &[SET_GLADIATOR_BEAST])
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 100, tp, true, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_turn_player(tp)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| deckspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_turn_player(1 - tp)
        && duel.is_existing_matching_card(|c, duel| extraspfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let (op, sum_loc) = if b1 { (1, LOCATION_DECK) } else { (2, LOCATION_EXTRA) };
    e.set_labels(duel, &[op, sum_loc.into()]);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, sum_loc as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let &[op, sum_loc] = e.get_labels(duel) else { return Ok(()) };
    if op == 1 && duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let spfilter = if op == 1 { deckspfilter } else { extraspfilter };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, sum_loc as u32, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.special_summon(tc, 100, tp, tp, op == 2, false, POS_FACEUP)?;
        let code = tc.get_original_code(duel);
        tc.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 0);
    }
    Ok(())
}
