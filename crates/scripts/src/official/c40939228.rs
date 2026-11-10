// Shooting Majestic Star Dragon
use crate::prelude::*;
use crate::proc_synchro::Requirement;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 40939228;
const MAJESTIC_DRAGON: u32 = 21159309;

pub const LISTED_NAMES: &[u32] = &[MAJESTIC_DRAGON, CARD_STARDUST_DRAGON];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;
    const REQ2: Option<Requirement> = Some(matfilter);

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[MAJESTIC_DRAGON])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Two in the Extra Deck, Synchro Summoned with Majestic Dragon, Stardust Dragon and Flame Viper (Level 2) on the field, and
// Raigeki to send the opponent's Sangan to the Graveyard for its effect to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (MAJESTIC_DRAGON, MZONE),
    (MAJESTIC_DRAGON, MZONE),
    (CARD_STARDUST_DRAGON, MZONE),
    (CARD_STARDUST_DRAGON, MZONE),
    (2830619, MZONE),
    (2830619, MZONE),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, negcon);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    c.is_race_full(duel, RACE_DRAGON, Some(sc), SUMMON_TYPE_SYNCHRO as u64, tp)
        && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), SUMMON_TYPE_SYNCHRO as u64, tp)
}

fn matfilter(duel: &Duel, g: CardSet, sc: CardId, tp: u8) -> bool {
    g.is_exists(duel, |c, duel| cfilter(c, duel, sc, tp), 1, None)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let Some(tc) = duel
        .select_matching_card(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection_full(tc, true);
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    e2.set_value(duel, RESET_TURN_SET as i32);
    tc.register_effect(duel, e2);
    Ok(())
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_STARDUST_DRAGON) || (c.is_type(duel, TYPE_SYNCHRO) && c.lists_code(duel, CARD_STARDUST_DRAGON))
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(atkfilter, e.get_handler_player(duel), LOCATION_GRAVE, 0, None) as i32
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && (rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT) || !relation));
    }
    let mut rg = CardSet::from(c);
    if relation {
        rg.insert(rc);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && duel.remove(c, POS_FACEUP, REASON_EFFECT | REASON_TEMPORARY)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_label_object_group(duel, Some(c.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_operation(duel, retop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
            duel.remove(a.eg.unwrap_or_default(), POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(c) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(c)?;
    }
    Ok(())
}
