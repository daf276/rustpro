// Cooky★Yummy Way
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 67098897;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| !c.is_link_monster(duel))
    }

    fn sub1(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_link(duel, &[1])
    }
}

// In the Extra Deck, Synchro Summoned with Effect Veiler (Level 1 Tuner) and Marshmao (Level 1) or with Artemis, the
// Magistus Moon Maiden (Link-1); on the field with Marshmao in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (10966439, MZONE),
    (10966439, MZONE),
    (10966439, GRAVE),
    (97268402, MZONE),
    (34755994, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0a = duel.create_effect(c);
    e0a.set_type(duel, EFFECT_TYPE_FIELD);
    e0a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e0a.set_code(duel, EFFECT_SYNCHRO_LEVEL);
    e0a.set_range(duel, LOCATION_EXTRA);
    e0a.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e0a.set_target_card(duel, |duel, _, c| c.is_link(duel, &[1]));
    e0a.set_value(
        duel,
        Value::Card(|duel, _, sc| if sc.is_set_card(duel, &[SET_YUMMY]) { 1 } else { -1 }),
    );
    c.register_effect(duel, e0a);
    let e0b = e0a.clone_effect(duel);
    e0b.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e0b.set_value(duel, Value::Card(|duel, _, sc| i32::from(!sc.is_set_card(duel, &[SET_YUMMY]))));
    c.register_effect(duel, e0b);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_cost(duel, Cost::self_to_extra);
    e2.set_target(duel, gysptg);
    e2.set_operation(duel, gyspop);
    c.register_effect(duel, e2);
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_can_turn_set(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSITION);
    let g = duel.select_target(tp, CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 2, None)?;
    duel.set_operation_info(
        0,
        CATEGORY_POSITION,
        g,
        g.map_or(0, |g| g.len()) as u8,
        tp,
        POS_FACEDOWN_DEFENSE as i32,
    );
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.change_position(tg, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, e) = (a.tp, a.e);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, e) = (a.tp, a.e);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    ft = ft.min(2);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, ft as u8, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
