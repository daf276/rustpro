// Angelechy Destrier
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 55393975;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field with a monster of the opponent's in another column. Its effects as a Continuous Spell need Angelechy cards that place it.
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
    e3a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3a.set_code(duel, EVENT_CHAINING);
    e3a.set_range(duel, LOCATION_STZONE);
    e3a.set_condition(duel, |duel, a| a.ep == 1 - a.tp && a.e.get_handler(duel).is_continuous_spell(duel));
    e3a.set_operation(duel, |duel, a| {
        a.e.get_handler(duel)
            .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1);
        Ok(())
    });
    c.register_effect(duel, e3a);
    let e3b = duel.create_effect(c);
    e3b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3b.set_code(duel, EVENT_CHAIN_SOLVED);
    e3b.set_range(duel, LOCATION_STZONE);
    e3b.set_condition(duel, |duel, a| a.ep == 1 - a.tp && a.e.get_handler(duel).has_flag_effect(duel, ID));
    e3b.set_operation(duel, |duel, a| {
        duel.hint(HINT_CARD, 0, u64::from(ID));
        duel.damage(1 - a.tp, 500, REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e3b);
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let colg = c.get_column_group(duel).union(c.into());
    let filter = |c: CardId, duel: &Duel| c.is_able_to_remove(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && !colg.is_contains(chkc) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(colg)));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(colg))?;
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
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            a.e.get_handler(duel).is_continuous_spell(duel) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
