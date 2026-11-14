// D - Burst
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 47471667;

pub const LISTED_SERIES: &[u16] = &[SET_DESTINY_HERO];

const DOGMA: u32 = 17132130;

pub const LISTED_NAMES: &[u32] = &[DOGMA];

// In the hand with Axe of Despair (a face-up Spell to destroy, and an Equip Card for the second attack) and Destiny HERO monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (40619825, HAND),
    (40619825, HAND),
    (13093792, MZONE),
    (13093792, HAND),
    (16605586, GRAVE),
    (99171160, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, atkcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, |duel, _| {
        duel.chain_attack();
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn spellfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_on_field(duel) && chkc.is_spell(duel) && chkc.is_faceup(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(spellfilter, tp, LOCATION_ONFIELD, 0, 1, None) && duel.is_player_can_draw(tp, 1));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, spellfilter, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED) as i32,
    );
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO])
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let locations = LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED;
    if tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && duel.draw(tp, 1, REASON_EFFECT)? > 0
        && duel.get_location_count_full(tp, LOCATION_MZONE, 0, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DESTINY_HERO])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
        && duel.is_existing_matching_card(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            locations,
            0,
            1,
            None,
        )
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            locations,
            0,
            1,
            1,
            None,
        )?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn atkcon(duel: &Duel, _: &Args) -> bool {
    let Some(ac) = duel.get_attacker() else { return false };
    (ac.has_equip_card(duel) || ac.is_code(duel, DOGMA)) && duel.check_mut(|duel| ac.can_chain_attack(duel))
}
