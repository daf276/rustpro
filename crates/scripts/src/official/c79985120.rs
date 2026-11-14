// Raidraptor - Revolution Falcon - Air Raid
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 79985120;

pub const LISTED_SERIES: &[u16] = &[SET_RANK_UP_MAGIC, SET_RAIDRAPTOR];
pub const LISTED_NAMES: &[u32] = &[81927732];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_WINGEDBEAST, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(6);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let g = duel.get_matching_group(cfilter, tp, LOCATION_HAND, 0, None);
        let cancelable = Xyz::proc_cancellable(duel);
        let Some(tc) = g.select_unselect(duel, Some(CardSet::default()), tp, false, cancelable, 1, 1)? else {
            return Ok(false);
        };
        duel.send_to_grave(tc, REASON_DISCARD | REASON_COST)?;
        Ok(true)
    }
}

// In the Extra Deck, Xyz Summoned with three Level 6 Winged Beasts or over Rise Falcon (Rank 4) by discarding a Rank-Up-Magic Spell, with
// Sangan on the opponent's side to destroy. Revolution Falcon is not ported, so the Special Summon from the Graveyard is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (35712107, MZONE),
    (35712107, MZONE),
    (49858495, MZONE),
    (73887236, MZONE),
    (3298689, HAND),
    (62104532, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RANK_UP_MAGIC]) && c.is_spell(duel) && c.is_discardable(duel)
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let sumtype = u64::from(SUMMON_TYPE_XYZ);
    c.is_faceup(duel)
        && c.is_set_card_full(duel, &[SET_RAIDRAPTOR], Some(xyzc), sumtype, tp)
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), sumtype, tp)
        && c.is_rank_below(duel, 5)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel)
}

// The core rounds the Lua's `atk/2` where it reads it as an integer.
fn half_attack(duel: &Duel, c: CardId) -> i32 {
    (c.get_attack(duel) + 1) / 2
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let tc = g.and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(false) };
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, half_attack(duel, tc));
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let mut dam = tc.get_attack(duel);
        if dam < 0 || tc.is_facedown(duel) {
            dam = 0;
        }
        if duel.destroy(tc, REASON_EFFECT)? != 0 {
            duel.damage(1 - tp, dam as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_controler(duel, a.tp) && a.rp == 1 - a.tp && c.is_reason(duel, REASON_DESTROY)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 81927732)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? != 0
        && c.is_relate_to_effect(duel, e)
    {
        duel.overlay(tc, c)?;
    }
    Ok(())
}
