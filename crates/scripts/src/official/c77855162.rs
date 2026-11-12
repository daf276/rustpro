// Tilting Entrainment
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 77855162;

// In the Extra Deck, Synchro Summoned with Junk Synchron (a Level 3 Tuner) and Shapesnatch (Level 5), with Flash Knight (a Level 4 Pendulum
// Normal Monster) in hand to summon and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (63977008, MZONE),
    (63977008, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (17390179, HAND),
    (17390179, HAND),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition_mut(duel, descon1);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e2b = e2.clone_effect(duel);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_condition_mut(duel, descon2);
    c.register_effect(duel, e2b);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_type(duel, TYPE_PENDULUM) && c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_HAND) {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
    } else {
        c.is_faceup(duel) && duel.get_location_count_from_ex(tp, tp, None, c) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn battled_monster(duel: &mut Duel, a: &Args) -> Option<(CardId, CardId)> {
    let (x, b) = duel.get_battle_monster(a.tp);
    a.e.set_label_object_group(duel, b.map(CardSet::from));
    let (x, b) = (x?, b?);
    (b.is_relate_to_battle(duel) && b.is_location(duel, LOCATION_MZONE) && x.is_status(duel, STATUS_OPPO_BATTLE)).then_some((x, b))
}

fn descon1(duel: &mut Duel, a: &Args) -> bool {
    battled_monster(duel, a).is_some()
}

fn descon2(duel: &mut Duel, a: &Args) -> bool {
    battled_monster(duel, a).is_some_and(|(x, _)| x.is_faceup(duel) && x.is_type(duel, TYPE_PENDULUM))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let targets = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, targets, 1, a.tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && tc.is_relate_to_battle(duel)
        && tc.is_controler(duel, 1 - tp)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
