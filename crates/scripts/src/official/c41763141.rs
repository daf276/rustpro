// Fata Dragna
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 41763141;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(matfilter); 2];
}

struct SendToDeck;

impl aux::DelayedOperation for SendToDeck {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_deck(ag, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        Ok(())
    }
}

// A copy on the field to be Fused by Polymerization with a WIND Dragon (Luster Dragon, Baby Dragon: the same Type and
// Attribute, different Levels) into one of two in the Extra Deck, the other to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (11091375, HAND),
    (11091375, HAND),
    (88819587, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_fusion_monster(duel));
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE + CATEGORY_TOEXTRA);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r & REASON_FUSION == REASON_FUSION
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, sump: u8, sg: Option<CardSet>) -> bool {
    if !c.has_level(duel) {
        return false;
    }
    let Some(sg) = sg else { return true };
    let others = Some(c.into());
    sg.filter_count(duel, aux::TRUE, others) == 0
        || (sg.is_exists(
            duel,
            |sc, duel| sc.is_attribute_full(duel, c.get_attribute(duel), Some(fc), sumtype, sump),
            1,
            others,
        ) && sg.is_exists(
            duel,
            |sc, duel| sc.is_race_full(duel, c.get_race(duel), Some(fc), sumtype, sump),
            1,
            others,
        ) && !sg.is_exists(duel, |sc, duel| sc.is_level(duel, &[c.get_level(duel)]), 1, others))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_fusion_monster(duel)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
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
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sc = duel
        .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        sc.negate_effects(duel, c, 0, false);
        aux::delayed_operation_full::<SendToDeck>(duel, sc.into(), PHASE_END, 0, ID, e, tp, aux::stringid(ID, 1), 0);
    }
    duel.special_summon_complete()?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_fusion_monster(duel)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
