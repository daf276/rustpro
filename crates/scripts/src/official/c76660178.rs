// Fisherman of Legend
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 76660178;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI, CARD_DARK_TIME_WIZARD, ID];

struct ReturnOperation;

impl aux::DelayedOperation for ReturnOperation {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(ag, PLAYER_NONE, REASON_EFFECT)?;
        Ok(())
    }
}

// In the hand, with Panthers on the field to meet its condition and the opponent's Elfnote Lucina to attack and be destroyed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (77482666, MZONE),
    (77482666, MZONE),
    (77482666, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::CardHint(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_CHAINING);
    e1a.set_range(duel, LOCATION_HAND);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp
            && a.re.is_some_and(|re| re.is_monster_effect(duel))
            && duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_TRIGGERING_LOCATION) == LOCATION_MZONE
    });
    e1a.set_target(duel, sptg);
    e1a.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1b.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - a.tp))
    });
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_UMI) || (c.lists_code(duel, CARD_DARK_TIME_WIZARD) && !c.is_code(duel, ID))) && c.is_faceup(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(spconfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        aux::delayed_operation_hint::<ReturnOperation>(duel, c.into(), PHASE_END, ID, e, tp, aux::stringid(ID, 2));
    }
    Ok(())
}
