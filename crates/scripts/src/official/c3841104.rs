// Destratification Dino Stratiae
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 3841104;
const BRACCHIO_RAIDUS: u32 = 16507828;
const ATTACK: u8 = 0;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_race_full(duel, RACE_DINOSAUR, Some(lc), sumtype, tp), 1, None)
    }
}

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DINOSAUR)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, None);
        }
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
        }
        Ok(true)
    }

    fn stage2(duel: &mut Duel, e: EffectId, _: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if chk == 2 {
            let c = e.get_handler(duel);
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 2));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e1.set_target_range(duel, 1, 0);
            e1.set_target_card(duel, |duel, _, c| {
                c.is_location(duel, LOCATION_EXTRA) && c.is_race_except(duel, RACE_DINOSAUR)
            });
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
            aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_DINOSAUR));
        }
        Ok(())
    }
}

// In the Extra Deck, Link Summoned with Two-Headed King Rex and Crawling Dragon #2 (Dinosaurs), with Bracchio-raidus in the Extra
// Deck to Fusion Summon from the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (BRACCHIO_RAIDUS, EXTRA),
    (94119974, MZONE),
    (94119974, MZONE),
    (38289717, MZONE),
    (38289717, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(BRACCHIO_RAIDUS),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_label_object(duel, Some(e0));
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_link_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e0| e0.get_label(duel) > 0)
    });
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let tp = e.get_handler_player(duel);
    let mg = c.get_material(duel).filter(
        duel,
        |mc, duel| mc.is_race_full(duel, RACE_DINOSAUR, Some(c), SUMMON_TYPE_LINK as u64, tp),
        None,
    );
    e.set_label(duel, mg.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))).div_euclid(2));
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let atk = e.get_label_object(duel).map_or(0, |e0| e0.get_label(duel));
    e.set_chain_data(duel, ATTACK, atk);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(e.get_handler(duel).into()), 1, tp, atk as i32);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        c.update_attack(duel, e.get_chain_data(duel, ATTACK).unwrap_or(0) as i32, None, None, None);
    }
    Ok(())
}
