// Destiny HERO - Dreadmaster
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT};

const ID: u32 = 40591390;
const CLOCK_TOWER_PRISON: u32 = 75041269;

pub const LISTED_NAMES: &[u32] = &[CLOCK_TOWER_PRISON];

// Clock Tower Prison, which has to be destroyed with 4 counters to Special Summon the card from the hand or Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (CLOCK_TOWER_PRISON, 0),
    (CLOCK_TOWER_PRISON, 0),
    (CLOCK_TOWER_PRISON, OPPONENT),
    (ID, HAND | OPPONENT),
    (ID, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| {
            let rc = re.get_handler(duel);
            rc.is_code(duel, CLOCK_TOWER_PRISON) || rc.is_code(duel, 4663194)
        })
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE);
    e3a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3a.set_code(duel, EFFECT_SET_ATTACK);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EFFECT_SET_DEFENSE);
    c.register_effect(duel, e3b);
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let g = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DESTINY_HERO])),
        c.get_controler(duel),
        LOCATION_MZONE,
        0,
        Some(c.into()),
    );
    g.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))) as i32
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || !c.is_set_card(duel, &[SET_DESTINY_HERO])
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, 0, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, 0, None);
    let break_chk = !g.is_empty();
    if break_chk {
        duel.destroy(g, REASON_EFFECT)?;
    }
    let mut mmz_count = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    if mmz_count > 0
        && duel.is_existing_matching_card(&filter, tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        mmz_count = mmz_count.min(2);
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            mmz_count = 1;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, &filter, tp, LOCATION_GRAVE, 0, 1, mmz_count as u8, None)?;
        if !sg.is_empty() {
            if break_chk {
                duel.break_effect()?;
            }
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_DESTINY_HERO]));
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    duel.register_effect(e2, tp);
    Ok(())
}
