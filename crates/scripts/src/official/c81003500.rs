// Elemental HERO Necroid Shaman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 81003500;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(86188410), FusionMaterial::Code(89252153)];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Elemental HERO Wildheart and Necroshade, to
// destroy the opponent's Sangan and Special Summon a monster from their Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (86188410, HAND),
    (89252153, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_controler(duel, 1 - tp)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, 1 - tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
