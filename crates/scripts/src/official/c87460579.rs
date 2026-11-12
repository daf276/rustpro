// Flower Cardian Lightflare
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 87460579;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 4;
    const MAX2: usize = 4;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];
// Three in the Extra Deck, Synchro Summoned with Flower Cardian Moonflowerviewing (a Level 6 Tuner) and four Flower Cardian Pine
// (Level 1), with Dark Hole in the opponent's Deck to negate and Elfnote Lucina to attack it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (33541430, MZONE),
    (81752019, MZONE),
    (81752019, MZONE),
    (81752019, MZONE),
    (81752019, MZONE),
    (53129443, DECK | OPPONENT),
    (53129443, DECK | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, discon);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition_mut(duel, discon2);
    e2.set_operation(duel, disop2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_condition(duel, sccon);
    e3.set_target(duel, sctg);
    e3.set_operation(duel, scop);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    let Some(re) = a.re else { return false };
    a.rp != a.tp && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && duel.is_chain_negatable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn discon2(duel: &mut Duel, a: &Args) -> bool {
    let tp = a.tp;
    let tc = duel.get_attacker();
    unported!(duel, tc.is_none());
    let Some(mut tc) = tc else { return false };
    let Some(mut oc) = tc.get_battle_target(duel) else { return false };
    if oc.is_controler(duel, tp) {
        (tc, oc) = (oc, tc);
    }
    a.e.set_label_object_group(duel, Some(oc.into()));
    tc.is_set_card(duel, &[SET_FLOWER_CARDIAN]) && tc.is_controler(duel, tp) && oc.is_controler(duel, 1 - tp)
}

fn disop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    (c.is_reason(duel, REASON_BATTLE)
        || (c.get_reason_player(duel) == 1 - tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, tp)))
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_synchro_summoned(duel)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FLOWER_CARDIAN])
        && c.is_type(duel, TYPE_SYNCHRO)
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
