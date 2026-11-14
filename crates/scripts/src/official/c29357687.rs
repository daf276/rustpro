// Gorgon, Empress of the Evil Eyed
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 29357687;

pub const LISTED_SERIES: &[u16] = &[SET_EVIL_EYE];

const CATOBLEPAS: u32 = 45955628;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_EVIL_EYE], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_EVIL_EYE_SELENE];

// Link Summoned from Catoblepas, or on the field with Evil Eye of Selene to equip and a monster of the opponent to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CATOBLEPAS, MZONE),
    (CATOBLEPAS, MZONE),
    (CATOBLEPAS, GRAVE),
    (CARD_EVIL_EYE_SELENE, HAND),
    (CARD_EVIL_EYE_SELENE, DECK),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = duel.get_matching_group(
        |c, duel| c.is_set_card(duel, &[SET_EVIL_EYE]),
        e.get_handler_player(duel),
        LOCATION_GRAVE,
        0,
        None,
    );
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32 * 100
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let eg = a.e.get_handler(duel).get_equip_group(duel);
    !eg.is_empty() && eg.is_exists(duel, |c, duel| c.is_code(duel, CARD_EVIL_EYE_SELENE), 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EFFECT) && c.is_negatable_monster(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY;
    let c = e.get_handler(duel);
    if duel.is_phase(PHASE_STANDBY) {
        c.register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_OATH, 2, i64::from(duel.get_turn_count()), 0);
    } else {
        c.register_flag_effect(duel, ID, reset, EFFECT_FLAG_OATH, 1);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if ((tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER)) && tc.is_relate_to_effect(duel, a.e) {
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

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_flag_effect_label(duel, ID)
        .is_some_and(|label| label != i64::from(duel.get_turn_count()))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(lg), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let mut g = c.get_linked_group(duel);
    if !g.is_empty() {
        if g.len() > 1 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            g = g.select(duel, tp, 1, 1, None)?;
        }
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
