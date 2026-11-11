// Sky Striker Ace - Camellia
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 63013339;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

struct ReturnControl;

impl aux::DelayedOperation for ReturnControl {
    fn operate(duel: &mut Duel, _: CardSet, a: &Args) -> Script<()> {
        let c = a.e.get_owner(duel);
        c.reset_effect(duel, EFFECT_SET_CONTROL, RESET_CODE);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_CONTROL);
        e1.set_value(duel, i32::from(c.get_owner(duel)));
        e1.set_reset(
            duel,
            RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_TEMP_REMOVE | RESET_TURN_SET)),
        );
        c.register_effect(duel, e1);
        Ok(())
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Extra Deck, Link Summoned with Sky Striker Ace - Raye and Roze, and on the field, with a Sky Striker card in the
// Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, MZONE), (26077387, MZONE), (37351133, MZONE), (98338152, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_sp_summon_once(duel, ID);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_value(duel, Value::SpSummonCondition(aux::lnklimit));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, tgcon);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.get_matching_group_count(CardId::is_spell, a.tp, LOCATION_GRAVE, 0, None) <= 3
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SKY_STRIKER]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_grave(duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, None)
            && duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, 1 - tp, false, false, POS_FACEUP)? > 0 {
        aux::delayed_operation::<ReturnControl>(duel, c.into(), PHASE_END, ID, e, tp);
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
        {
            duel.send_to_grave(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}
