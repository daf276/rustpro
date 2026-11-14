// Darklord Uprising
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 54527349;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const EVENINGSTAR: u32 = 10136446;

// The cost card a cost check's parameters exclude from the materials.
const COST_CARD: u64 = (ID as u64) << 16;

struct CostParams;

impl FusionSummonParams for CostParams {
    const CHKF: Option<u64> = Some(PLAYER_NONE as u64);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        duel.get_script_group(COST_CARD).is_none_or(|exc| !exc.is_contains(c)) && c.is_able_to_grave(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let exc = duel.get_script_group(COST_CARD).and_then(|g| g.get_first());
        (None, exc.map(|exc| -> FusionCheck { (costcardcheck, exc) }))
    }
}

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK)
    }

    fn stage2(duel: &mut Duel, e: EffectId, _: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        stage2(duel, e, tp, chk)
    }
}

// Darklord monsters in the hand and on the field to send and to fuse into Darklord Eveningstar.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (EVENINGSTAR, EXTRA),
    (EVENINGSTAR, EXTRA),
    (10426067, HAND),
    (84031359, HAND),
    (84031359, MZONE),
    (55690251, HAND),
    (55690251, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(EVENINGSTAR)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn costcardcheck(duel: &Duel, exc: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    duel.get_location_count_from_ex(tp, tp, Some(sg.union(exc.into())), fc) > 0
}

fn costfilter(c: CardId, duel: &Duel, a: &Args) -> bool {
    if !(c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel))
    {
        return false;
    }
    duel.check_mut(|duel| {
        duel.set_script_group(COST_CARD, Some(c.into()));
        Fusion::new_summon_eff_closure(duel, a.e, false);
        let res = Fusion::summon_eff_tg::<CostParams>(duel, a, 0, None).unwrap_or(false);
        duel.set_script_group(COST_CARD, None);
        res
    })
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, 100);
    let filter = |c, duel: &Duel| costfilter(c, duel, a);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    e.set_label_object_group(duel, g.get_first().map(CardSet::from));
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let e = a.e;
    if chk == 0 {
        if e.get_label(duel) == 100 {
            return Ok(true);
        }
        Fusion::new_summon_eff_closure(duel, e, false);
        return Fusion::summon_eff_tg::<Params>(duel, a, chk, chkc);
    }
    if let Some(cost_card) = e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        let atk = cost_card.get_base_attack(duel);
        duel.set_operation_info(0, CATEGORY_RECOVER, None, 1, a.tp, atk);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    Fusion::new_summon_eff_closure(duel, a.e, true);
    Fusion::summon_eff_op::<Params>(duel, a)
}

fn stage2(duel: &mut Duel, e: EffectId, tp: u8, chk: i32) -> Script<()> {
    if chk == 1
        && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && let Some(cost_card) = e.get_label_object_group(duel).and_then(|g| g.get_first())
    {
        let lp = cost_card.get_attack(duel);
        if lp > 0 && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            duel.break_effect()?;
            duel.recover(tp, lp as u32, REASON_EFFECT)?;
        }
        e.set_label_object_group(duel, None);
    }
    Ok(())
}
