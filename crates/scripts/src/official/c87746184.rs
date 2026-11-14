// Albion the Branded Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 87746184;

pub const LISTED_SERIES: &[u16] = &[SET_BRANDED];
pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ, ID];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_ALBAZ),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), sumtype, tp)),
    ];
}

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_level_below(duel, 8) && !c.is_code(duel, ID)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_none() {
            let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, |c, duel| c.is_able_to_remove(duel));
            return (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None);
        }
        (None, None)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let locations = LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE;
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, a.tp, locations as i32);
        Ok(false)
    }
}

// Fusion Summoned by Polymerization, so that its effect Fusion Summons Magistus Chorozo with it and a Spellcaster.
pub const SUPPORT: &[(u32, u16)] = &[(68468459, HAND), (24094653, HAND), (38142739, HAND), (66532962, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);

    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thsetcon);
    e2.set_target(duel, thsettg);
    e2.set_operation(duel, thsetop);
    c.register_effect(duel, e2);
}

fn thsetcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_turn_id(duel) == duel.get_turn_count() && !c.is_reason(duel, REASON_RETURN)
}

fn thsetfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BRANDED]) && c.is_spell_trap(duel) && (c.is_able_to_hand(duel) || c.is_ssetable(duel))
}

fn thsettg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thsetfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thsetop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let Some(sc) = duel
        .select_matching_card(tp, thsetfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| sc.is_ssetable(duel),
        |duel| duel.sset(tp, sc).map(|_| ()),
        aux::stringid(ID, 3),
    )
}
