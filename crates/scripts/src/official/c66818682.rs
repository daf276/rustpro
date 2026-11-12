// Moon Dragon Quilla
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 66818682;

pub const MATERIAL: &[u32] = &[78552773];
pub const LISTED_NAMES: &[u32] = &[78552773, 39823987];

// In the Extra Deck, Synchro Summoned from Supay (Level 1 Tuner) and Shapesnatch (Level 5), with Sun Dragon Inti in the
// Graveyard to revive, attackers on the opponent's side and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (78552773, MZONE),
    (78552773, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (39823987, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[78552773])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(tc) = duel.get_attacker() else { return Ok(false) };
    tc.create_effect_relation(duel, a.e);
    let atk = tc.get_attack(duel);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, (atk + 1) / 2);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_attacker()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let atk = (tc.get_attack(duel) + 1) / 2;
        duel.recover(a.tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 39823987) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
