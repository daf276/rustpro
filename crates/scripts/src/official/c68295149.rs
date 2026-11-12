// Linkmail Archfiend
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 68295149;
const TYPE_FRSX: u32 = TYPE_FUSION + TYPE_RITUAL + TYPE_SYNCHRO + TYPE_XYZ;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_FRSX, Some(lc), sumtype, tp), 1, None)
    }
}

// The destruction replacement banishes a Synchro Monster from the Graveyard (it moves no card of its own, so its prompt is the check).
// Link Summoned with a Synchro Monster (Cork Shooter) as material, with Synchro Monsters in the Graveyard to target and banish, and the
// opponent's monsters to weaken and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (23361526, MZONE),
    (23361526, MZONE),
    (62327910, MZONE),
    (62327910, MZONE),
    (23361526, GRAVE),
    (30655537, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_summon_location(duel, LOCATION_EXTRA));
    e2.set_value(duel, Value::EffectPlayer(tgval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, reptg);
    e3.set_operation(duel, repop);
    c.register_effect(duel, e3);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FRSX) && c.get_attack(duel) > 0 && (c.is_faceup(duel) || c.is_location(duel, LOCATION_GRAVE))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE | LOCATION_MZONE) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None)
            && duel.is_existing_target(atkfilter, tp, LOCATION_GRAVE | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, atkfilter, tp, LOCATION_GRAVE | LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && !g.is_empty()
    {
        let atk = i64::from(tc.get_attack(duel));
        for target in g {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, -atk as i32);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            target.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn tgval(duel: &Duel, e: EffectId, re: EffectId, rp: u8) -> bool {
    re.is_monster_effect(duel) && rp != e.get_handler_player(duel)
}

fn repfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FRSX) && c.is_monster(duel) && c.is_able_to_remove(duel)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && c.is_location(duel, LOCATION_MZONE)
            && c.is_faceup(duel)
            && duel.is_existing_matching_card(repfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
        let g = duel.select_matching_card(tp, repfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
        e.set_label_object_group(duel, Some(g));
        return Ok(true);
    }
    Ok(false)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.remove(tc, POS_FACEUP, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}
