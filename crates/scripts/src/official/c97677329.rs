// Backup Supervisor
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 97677329;
const BACKUP_SECRETARY: u32 = 63528891;

pub const LISTED_NAMES: &[u32] = &[BACKUP_SECRETARY];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// In the Extra Deck, Link Summoned with Backup Secretary and a Cyberse monster in the hand to Special Summon, and Backup
// Secretary in the Deck to replace it when the opponent destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (BACKUP_SECRETARY, MZONE),
    (BACKUP_SECRETARY, MZONE),
    (BACKUP_SECRETARY, MZONE),
    (24154052, MZONE),
    (BACKUP_SECRETARY, DECK),
    (BACKUP_SECRETARY, DECK),
    (47226949, HAND),
    (47226949, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon1);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon2);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let tp = c.get_controler(duel);
    let found = g.is_exists(
        duel,
        |mc, duel| mc.is_summon_code(duel, Some(c), u64::from(SUMMON_TYPE_LINK), tp, &[BACKUP_SECRETARY]),
        1,
        None,
    );
    e.set_label(duel, i64::from(found));
}

fn spcon1(duel: &Duel, a: &Args) -> bool {
    if a.e.get_label_object(duel).is_none_or(|e0| e0.get_label(duel) != 1) {
        return false;
    }
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    let Some(mut attacker) = duel.get_attacker() else { return false };
    let Some(mut defender) = attacker.get_battle_target(duel) else {
        return false;
    };
    if attacker.is_controler(duel, 1 - a.tp) {
        std::mem::swap(&mut attacker, &mut defender);
    }
    lg.is_contains(attacker)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE) || (a.rp == 1 - a.tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, a.tp))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, BACKUP_SECRETARY) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
