// Enneacraft - Asta.PIXEA
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 28454232;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone and face-down on the field, with the opponent's Mage Power to target and Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_destroy::<ID>(duel, c);
    register_set_summon::<ID>(duel, c);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_DISABLE + CATEGORY_REMOVE, 0);
    e4.set_condition(duel, discon);
    e4.set_target(duel, distg);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
    register_flip_flag::<ID>(duel, c);
    let e5b = duel.create_effect(c);
    e5b.set_type(duel, EFFECT_TYPE_FIELD);
    e5b.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE);
    e5b.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e5b.set_range(duel, LOCATION_MZONE);
    e5b.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e5b.set_target_range(duel, LOCATION_ONFIELD | LOCATION_GRAVE, LOCATION_ONFIELD | LOCATION_GRAVE);
    e5b.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e5b);
}

// The Pendulum procedure and the E.C. Counter placed on each flip, which every Enneacraft Pendulum Monster opens with.
pub fn register_counter(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit_full(duel, COUNTER_EC, LOCATION_PZONE as u16);
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_FLIP);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_operation(duel, counterop);
    c.register_effect(duel, e1);
}

fn counterop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !duel.is_chain_solving() {
        c.add_counter(duel, COUNTER_EC, 1);
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).add_counter(duel, COUNTER_EC, 1);
        Ok(())
    });
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_label_object(duel, Some(e1));
    e2.set_operation(duel, |duel, a| {
        if let Some(e1) = a.e.get_label_object(duel) {
            e1.reset(duel);
        }
        Ok(())
    });
    e2.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e2, a.tp);
    Ok(())
}

// The Battle Phase destruction of the Level 9 Enneacraft monsters.
pub fn register_destroy<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let handler = a.e.get_handler(duel);
    duel.get_first_matching_card(aux::TRUE, a.tp, LOCATION_PZONE, 0, Some(handler.into()))
        .is_some_and(|pc| pc.is_set_card(duel, &[SET_ENNEACRAFT]))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let atk = a.e.get_handler(duel).get_attack(duel) - 1;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_faceup(duel)
            && chkc.is_attack_below(duel, atk));
    }
    let filter = aux::faceup_filter(move |c, duel| c.is_attack_below(duel, atk));
    if chk == 0 {
        return Ok(duel.is_existing_target(&filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, &filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

// The Special Summon from the hand in face-down Defense Position every Enneacraft Pendulum Monster has.
pub fn register_set_summon<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::self_reveal, spcost::<ID>]));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| c.is_position(duel, POS_FACEDOWN_DEFENSE)),
    );
}

fn spcost<const ID: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 4));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_FORCE_SPSUMMON_POSITION);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, POS_FACEDOWN_DEFENSE as i32);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn set_filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| set_filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| set_filter(c, duel, a.e, a.tp),
        LOCATION_HAND,
        POS_FACEDOWN_DEFENSE,
    )
}

// The Quick Effect a face-down Enneacraft monster activates by flipping itself face-up; the card sets what it does.
pub fn flip_quick<const ID: u32>(duel: &mut Duel, c: CardId, category: u64, property: u32) -> EffectId {
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, category);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + property);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, Cost::self_change_position::<POS_FACEUP_DEFENSE>);
    e4
}

// The flag that marks the Level 9 Enneacraft monsters flipped face-up, which their lasting effects check.
pub fn register_flip_flag<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e5a = duel.create_effect(c);
    e5a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e5a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e5a.set_code(duel, EVENT_FLIP);
    e5a.set_operation(duel, |duel, a| {
        let reset = RESET_EVENT | RESETS_STANDARD;
        a.e.get_handler(duel)
            .register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_CLIENT_HINT, 1, 0, aux::stringid(ID, 3));
        Ok(())
    });
    c.register_effect(duel, e5a);
}

fn disconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_on_field(duel) && c.is_controler(duel, tp)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    if !(a.rp == 1 - a.tp
        && a.e.get_handler(duel).is_facedown(duel)
        && a.re.is_some_and(|re| re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET))
        && duel.is_chain_disablable(ev))
    {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(ev, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, |c, duel| disconfilter(c, duel, a.tp), 1, None))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, a.tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.negate_effect(a.ev as u8) {
        return Ok(());
    }
    let g = duel.get_matching_group(
        |c, duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT),
        tp,
        0,
        LOCATION_HAND,
        None,
    );
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 5))? {
        return Ok(());
    }
    let rg = g.random_select(duel, tp, 1);
    if !rg.is_empty() {
        duel.remove(rg, POS_FACEDOWN, REASON_EFFECT)?;
    }
    Ok(())
}
