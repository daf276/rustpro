// Angelechy Shatranga
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 42410161;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field with a monster of the opponent's to banish. Its effects as a Continuous Spell need Angelechy cards that place it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (15025844, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_MOVE);
    e2.set_range(duel, LOCATION_STZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3a.set_code(duel, EVENT_CHAINING);
    e3a.set_range(duel, LOCATION_STZONE);
    e3a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_continuous_spell(duel));
    e3a.set_operation(duel, |duel, a| {
        if a.ep == a.tp || !a.re.is_some_and(|re| re.is_monster_effect(duel)) {
            return Ok(());
        }
        let reset = RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_CONTROL | RESET_PHASE | PHASE_END;
        a.e.get_handler(duel).register_flag_effect(duel, ID, reset, 0, 1);
        Ok(())
    });
    c.register_effect(duel, e3a);
    let e3b = duel.create_effect(c);
    e3b.set_type(duel, EFFECT_TYPE_FIELD);
    e3b.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3b.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3b.set_range(duel, LOCATION_STZONE);
    e3b.set_target_range(duel, 0, 1);
    e3b.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect_full(duel, ID, 5));
    e3b.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    c.register_effect(duel, e3b);
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = |c: CardId, duel: &Duel| c.is_able_to_remove(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.eg.is_some_and(|eg| eg.is_contains(c)) && !c.is_previous_location(duel, LOCATION_SZONE)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_continuous_spell(duel) && duel.is_existing_matching_card(thfilter, tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, locations as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        if sc.is_location(duel, LOCATION_GRAVE) {
            duel.hint_selection(sc);
        }
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        if sc.is_previous_location(duel, LOCATION_DECK) {
            duel.confirm_cards(1 - tp, sc)?;
        }
    }
    Ok(())
}
