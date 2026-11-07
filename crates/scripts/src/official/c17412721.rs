// Elder Entity Norden
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 17412721;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_SYNCHRO | TYPE_XYZ, Some(fc), sumtype, tp)); 2];
}

// Fusion Summoned by Polymerization with Chaos Angel and Crystal Wing Synchro Dragon, to Special Summon Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (22850702, MZONE),
    (50954680, MZONE),
    (26202165, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_EXTRA));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
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
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let c = e.get_handler(duel);
        tc.negate_effects(duel, c, 0, false);
        if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            c.set_card_target(duel, tc);
            c.create_relation(duel, tc, RESET_EVENT | RESET_OVERLAY | RESET_MSCHANGE | RESET_TURN_SET);
            tc.create_relation(duel, c, RESET_EVENT | RESETS_STANDARD | RESET_OVERLAY);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
            e1.set_code(duel, EVENT_LEAVE_FIELD_P);
            e1.set_operation(duel, leaveop);
            e1.set_label_object_group(duel, Some(tc.into()));
            e1.set_reset(duel, RESET_EVENT | (RESET_OVERLAY | RESET_MSCHANGE | RESET_TURN_SET));
            c.register_effect(duel, e1);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

// Registers the banishing effect only if Norden's effects aren't negated right before it leaves the field.
fn leaveop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_LEAVE_FIELD);
    e1.set_operation(duel, banishop);
    e1.set_label_object_group(duel, a.e.get_label_object_group(duel));
    e1.set_reset(duel, RESET_EVENT | (RESET_OVERLAY | RESET_MSCHANGE | RESET_TURN_SET));
    c.register_effect(duel, e1);
    Ok(())
}

fn banishop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first())
        && c.is_relate_to_card(duel, tc)
        && tc.is_relate_to_card(duel, c)
    {
        c.release_relation(duel, tc);
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    e.reset(duel);
    Ok(())
}
