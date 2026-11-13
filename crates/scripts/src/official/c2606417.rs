// Reversal Box
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 2606417;
const COUNTER_REVERSAL_BOX: u16 = 0x21b;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_TIME_WIZARD];
pub const TOSS_COIN: bool = true;

// Copies on the field to roll for counters, Elfnote Lucina on the opponent's side to attack and Dark Time Wizard monsters
// (Fisherman of Legend, Alligator's Dragon Knight) in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (76660178, DECK),
    (76660178, DECK),
    (4881365, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_REVERSAL_BOX);
    c.set_counter_limit(duel, COUNTER_REVERSAL_BOX, 6);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DICE + CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, countertg);
    e1.set_operation(duel, counterop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(
        duel,
        CATEGORY_COIN + CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE + CATEGORY_DISABLE,
    );
    e2a.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2a.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2a.set_code(duel, EVENT_CHAINING);
    e2a.set_range(duel, LOCATION_SZONE);
    e2a.set_condition(duel, coincon);
    e2a.set_cost(duel, Cost::remove_counter_from_self::<COUNTER_REVERSAL_BOX, 1>);
    e2a.set_target(duel, cointg);
    e2a.set_operation(duel, coinop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e2b.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - a.tp))
    });
    e2b.set_cost(duel, |duel, a, chk| {
        Cost::and(
            duel,
            a,
            chk,
            &[
                Cost::remove_counter_from_self::<COUNTER_REVERSAL_BOX, 1>,
                Cost::soft_once_per_battle::<ID>,
            ],
        )
    });
    c.register_effect(duel, e2b);
}

fn countertg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_COUNTER, None, 1, tp, i32::from(COUNTER_REVERSAL_BOX));
    Ok(false)
}

fn counterop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        let Some(&die_roll) = duel.toss_dice(tp, 1)?.first() else {
            return Ok(());
        };
        let mut ct = 1;
        if die_roll != 1 {
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
            ct = duel.announce_number_range(tp, 1, u32::from(die_roll))? as u16;
        }
        c.add_counter_full(duel, COUNTER_REVERSAL_BOX, ct, true);
    }
    Ok(())
}

fn coincon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let trig_ctrl: u8 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_CONTROLER);
    a.re.is_some_and(|re| {
        re.is_monster_effect(duel)
            && trig_ctrl == 1 - a.tp
            && trig_loc == LOCATION_MZONE
            && re.get_handler(duel).is_relate_to_effect(duel, re)
    })
}

fn cointg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let opp_card = if e.get_code(duel) == EVENT_CHAINING {
        a.eg.and_then(|eg| eg.get_first())
    } else {
        None
    };
    let Some(opp_card) = opp_card.or_else(|| duel.get_attacker()) else {
        unported!(duel, true);
        return Ok(false);
    };
    if chk == 0 {
        return Ok(opp_card.is_can_be_effect_target(duel, e));
    }
    duel.set_target_card(opp_card);
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, Some(opp_card.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, Some(opp_card.into()), 1, tp, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.lists_code(duel, CARD_DARK_TIME_WIZARD) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn coinop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.call_coin(tp)?
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        let tc = duel.get_first_target();
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty()
            && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
            && let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
        {
            let c = e.get_handler(duel);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_SET_ATTACK);
            e1.set_value(duel, 0);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
            tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
        }
    }
    Ok(())
}
