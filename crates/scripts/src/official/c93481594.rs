// Solfachord Happiness
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, PZONE};

const ID: u32 = 93481594;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_SOLFACHORD];
// DoSolfachord Cutia and FaSolfachord Fancia (Scales 8 and 5) in the Deck to add, and in the Pendulum Zones to Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (55226153, DECK),
    (55226153, DECK),
    (54100561, DECK),
    (54100561, DECK),
    (55226153, PZONE),
    (54100561, PZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct ExtraPendulumSummon;

impl ExtraPendulum for ExtraPendulumSummon {
    const DESC: u64 = aux::stringid(ID, 6);
    const OWNER: u32 = ID;

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_SOLFACHORD])
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_type(duel, TYPE_PENDULUM) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_type(duel, TYPE_PENDULUM) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn scale(c: CardId, duel: &Duel) -> i64 {
    i64::from(c.get_scale(duel))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let handler = e.get_handler(duel);
    let b1 = !duel.has_flag_effect(tp, ID + 1)
        && duel.is_existing_matching_card(
            |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
            tp,
            LOCATION_HAND,
            0,
            1,
            Some(handler.into()),
        )
        && duel
            .get_matching_group(thfilter, tp, LOCATION_DECK, 0, None)
            .get_class_count(duel, scale)
            >= 2;
    let b2 = !duel.has_flag_effect(tp, ID + 2) && Pendulum::player_can_gain_additional_pendulum_summon(duel, tp, ID);
    let b3 = !duel.has_flag_effect(tp, ID + 3)
        && duel.get_location_count(tp, LOCATION_MZONE) >= 2
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_PZONE, 0, 2, None);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    let op = op.map_or(0, |op| op as u32);
    e.set_label(duel, i64::from(op));
    duel.register_flag_effect(tp, ID + op, RESET_PHASE | PHASE_END, 0, 1);
    match op {
        1 => {
            e.set_category(duel, CATEGORY_HANDES + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
            duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
            duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
        }
        2 => e.set_category(duel, 0),
        3 => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_PZONE as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            if duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)? == 0 {
                return Ok(());
            }
            let dg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
            let sdg = aux::select_unselect_group(duel, dg, 2, 2, &aux::dpcheck(scale), tp, HINTMSG_ATOHAND)?;
            if sdg.len() != 2 || duel.send_to_hand(sdg, PLAYER_NONE, REASON_EFFECT)? != 2 {
                return Ok(());
            }
            let in_hand = sdg.filter(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
            duel.confirm_cards(1 - tp, in_hand)?;
            duel.shuffle_hand(tp)?;
            let ct = duel.get_location_count(tp, LOCATION_MZONE);
            if ct == 0 {
                return Ok(());
            }
            let hg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
            if hg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 4))? {
                return Ok(());
            }
            let mut ct = ct.min(duel.get_field_group_count(tp, 0, LOCATION_MZONE) as i32 + 1);
            if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
                ct = 1;
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let shg = hg.select(duel, tp, 1, ct as u8, None)?;
            if !shg.is_empty() {
                duel.break_effect()?;
                duel.special_summon(shg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        2 => {
            let handler = e.get_handler(duel);
            Pendulum::grant_additional_pendulum_summon::<ExtraPendulumSummon>(duel, handler, tp, aux::stringid(ID, 5));
        }
        3 => {
            if duel.get_location_count(tp, LOCATION_MZONE) < 2 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
                return Ok(());
            }
            let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_PZONE, 0, None);
            if g.len() == 2 {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        _ => {}
    }
    Ok(())
}
