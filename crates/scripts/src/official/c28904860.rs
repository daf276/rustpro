// Angelechy Bastion
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 28904860;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];

const CARD_ANGELECHY_SHATRANGA: u32 = 42410161;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_ANGELECHY_SHATRANGA];

// On the field with a card of the opponent's in its column. Its effects as a Continuous Spell need Angelechy cards that place it.
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
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_MOVE);
    e2.set_range(duel, LOCATION_STZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, plcon);
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_range(duel, LOCATION_STZONE);
    e3.set_target_range(duel, LOCATION_ONFIELD, LOCATION_ONFIELD);
    e3.set_target_card(duel, |duel, e, c| c != e.get_handler(duel) && c.is_set_card(duel, &[SET_ANGELECHY]));
    e3.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| rp == 1 - e.get_handler_player(duel)));
    c.register_effect(duel, e3);
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let colg = c.get_column_group(duel).filter(duel, |c, duel| c.is_able_to_remove(duel), None);
    if let Some(chkc) = chkc {
        return Ok(colg.is_contains(chkc) && chkc != c);
    }
    let targetable = |c: CardId, duel: &Duel| c.is_can_be_effect_target(duel, e);
    if chk == 0 {
        return Ok(colg.is_exists(duel, targetable, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = colg.filter_select(duel, tp, targetable, 1, 1, None)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn plcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.eg.is_some_and(|eg| eg.is_contains(c)) && !c.is_previous_location(duel, LOCATION_SZONE)
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_ANGELECHY_SHATRANGA) && !c.is_forbidden(duel)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let tp = a.tp;
        return Ok(a.e.get_handler(duel).is_continuous_spell(duel)
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(plfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
        sc.register_effect(duel, e1);
    }
    Ok(())
}
