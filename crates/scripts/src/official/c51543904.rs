// Number 99: Utopic Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 51543904;
pub const XYZ_NUMBER: u32 = 99;

// In the Extra Deck, Xyz Summoned over Number 39: Utopia by discarding Rank-Up-Magic Launch, with a Number in the Graveyard to
// summon, and the opponent's Blackwing - Gale the Whirlwind (a monster effect that targets) to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (84013237, MZONE),
    (84013237, MZONE),
    (84013237, GRAVE),
    (3298689, HAND),
    (3298689, HAND),
    (2009101, MZONE | OPPONENT),
    (2009101, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> =
        Some(|c, duel, tp, lc| c.is_faceup(duel) && c.is_set_card_full(duel, &[SET_UTOPIA], Some(lc), u64::from(SUMMON_TYPE_XYZ), tp));
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

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, stock::negate_destroy_target);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RANK_UP_MAGIC]) && c.is_spell(duel) && c.is_discardable(duel)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NUMBER]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| filter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(re) = a.re else { return false };
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) || !re.is_monster_effect(duel) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.contains(c)) && duel.is_chain_negatable(a.ev as u8)
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
