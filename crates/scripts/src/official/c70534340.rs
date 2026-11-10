// Lubellion the Searing Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 70534340;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_ALBAZ),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp)),
    ];
}

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_level_below(duel, 8) && !c.is_code(duel, ID)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_deck(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, |c, duel| c.is_faceup(duel) && c.is_able_to_deck(duel));
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None)),
            None,
        )
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn stage2(duel: &mut Duel, e: EffectId, _: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if chk == 2 {
            let c = e.get_handler(duel);
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3206);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CANNOT_ATTACK);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            c.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_description(duel, aux::stringid(ID, 1));
            e2.set_type(duel, EFFECT_TYPE_FIELD);
            e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e2.set_target_range(duel, 1, 0);
            e2.set_target_card(duel, |duel, _, c| {
                !c.is_type(duel, TYPE_FUSION) && c.is_location(duel, LOCATION_EXTRA)
            });
            e2.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e2, tp);
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(
            0,
            CATEGORY_TODECK,
            None,
            0,
            a.tp,
            (LOCATION_MZONE | LOCATION_GRAVE | LOCATION_REMOVED) as i32,
        );
        Ok(false)
    }
}

// In the Extra Deck, Fusion Summoned with Polymerization, Fallen of Albaz and a DARK monster, with Invoked Purgatrio in the Extra Deck
// and Aleister the Invoker and Darkfire Soldier #1 in the Graveyard to Fusion Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_ALBAZ, HAND),
    (26202165, HAND),
    (12307878, EXTRA),
    (86120751, GRAVE),
    (5388481, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::SpecialSummonedCode(12307878)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}
