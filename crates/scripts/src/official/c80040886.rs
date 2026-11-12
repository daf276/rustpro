// Navy Dragon Mech
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 80040886;

// In the Extra Deck, Synchro Summoned with Zeta (Level 4 Tuner) and Akihiron (Level 5), with Tuners in the Graveyard, the opponent's Sangan to negate and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (42026772, MZONE),
    (42026772, MZONE),
    (36904469, MZONE),
    (487395, GRAVE),
    (487395, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn is_tuner(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None)
            && duel.is_existing_matching_card(is_tuner, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let ct = duel.get_matching_group_count(is_tuner, tp, LOCATION_GRAVE, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let dg = duel.get_target_cards(a.e).unwrap_or_default();
    if dg.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in dg.iter() {
        if (tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER) {
            duel.negate_related_chain(tc, RESET_TURN_SET);
            super::c97268402::negate(duel, c, tc, EFFECT_FLAG_CANNOT_DISABLE);
            if tc.is_type(duel, TYPE_TRAPMONSTER) {
                let e3 = duel.create_effect(c);
                e3.set_type(duel, EFFECT_TYPE_SINGLE);
                e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
                e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
                tc.register_effect(duel, e3);
            }
        }
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_synchro_summoned(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}
