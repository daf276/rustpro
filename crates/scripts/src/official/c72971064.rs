// Lyrilusc - Ensemblue Robin
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72971064;

pub const LISTED_SERIES: &[u16] = &[SET_LYRILUSC];
// In the Extra Deck, Xyz Summoned with Level 1 Normal Monsters, with a Lyrilusc in the Graveyard and the opponent's Dark Hole to send it there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (1929294, MZONE),
    (3557275, MZONE),
    (48608796, GRAVE),
    (53129443, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 500));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, retopthcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, retopthtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_LEAVE_GRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_condition(duel, llthcon);
    e3.set_target(duel, llthtg);
    e3.set_operation(duel, llthop);
    c.register_effect(duel, e3);
}

fn retopthfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_summon_player(duel, 1 - tp) && c.is_able_to_hand(duel) && c.is_can_be_effect_target(duel, e) && c.is_location(duel, LOCATION_MZONE)
}

fn retopthcon(duel: &Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| retopthfilter(c, duel, e, tp), 1, None))
}

fn retopthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if let Some(chkc) = chkc {
        return Ok(eg.contains(chkc) && retopthfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| retopthfilter(c, duel, e, tp), 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let tc = if eg.len() == 1 {
        eg
    } else {
        eg.filter_select(duel, tp, |c, duel| retopthfilter(c, duel, e, tp), 1, 1, None)?
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tc), 1, 0, 0);
    Ok(false)
}

fn llthcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp == 1 - a.tp && c.get_previous_controler(duel) == c.get_owner(duel)
}

fn llthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LYRILUSC]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn llthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && llthfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(llthfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let tc = duel.select_target(tp, llthfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, tc, 1, 0, 0);
    Ok(false)
}

fn llthop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, a.tp, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
