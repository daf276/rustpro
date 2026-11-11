// Genia of the Ring
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 40041559;

// Fairy Tail - Sleeper to Tribute a copy on the field, with the opponent's Raigeki and Pot of Greed it reacts to, and the
// opponent's The Fallen & The Virtuous to destroy the protected monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (42921475, MZONE),
    (12580477, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
    (30271097, SZONE | OPPONENT),
    (30271097, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOHAND);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_RELEASE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_condition(duel, thcon);
    e2a.set_target(duel, thtg);
    e2a.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e2b);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        if tc.is_race_except(duel, RACE_SPELLCASTER) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_CHANGE_RACE);
            e1.set_value(duel, RACE_SPELLCASTER as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
        let e2 = duel.create_effect(c);
        e2.set_description(duel, aux::stringid(ID, 2));
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
        e2.set_count_limit(duel, 1, 0);
        e2.set_value(
            duel,
            Value::ReasonMut(|duel, e, _, r, _| {
                let by_effect = r & REASON_EFFECT > 0;
                if by_effect {
                    e.reset(duel);
                }
                by_effect
            }),
        );
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
    }
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 3));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_target_range(duel, 1, 0);
    e3.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && c.is_race_except(duel, RACE_SPELLCASTER)
    });
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !(a.e.get_handler(duel).is_reason(duel, REASON_COST) && re.is_activated(duel) && re.is_monster_effect(duel)) {
        return false;
    }
    let rc = re.get_handler(duel);
    if rc.is_relate_to_effect(duel, re) && rc.is_faceup(duel) {
        rc.is_race(duel, RACE_SPELLCASTER)
    } else {
        duel.get_chain_info::<u64>(0, CHAININFO_TRIGGERING_RACE) & RACE_SPELLCASTER > 0
    }
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
