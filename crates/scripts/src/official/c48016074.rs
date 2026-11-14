// Charmers of the Grand Circle
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 48016074;

pub const LISTED_SERIES: &[u16] = &[SET_CHARMER, SET_FAMILIAR_POSSESSED, SET_POSSESSED];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
        c.is_set_card_full(duel, &[SET_CHARMER, SET_FAMILIAR_POSSESSED], Some(fc), sumtype, tp)
    });
    const MINC: i32 = 2;
    const MAXC: i32 = 99;
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_CHARMER, SET_FAMILIAR_POSSESSED];

// In the Extra Deck, Fusion Summoned with Polymerization from Charmers (Hiita, Dharc, Wynn, Aussa), with Charmers in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (759393, HAND),
    (19327348, HAND),
    (37744402, MZONE),
    (37970940, MZONE),
    (759393, GRAVE),
    (19327348, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_ATKCHANGE + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON,
    );
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let count = c
        .get_material(duel)
        .get_class_count(duel, |c, duel| i64::from(c.get_original_attribute(duel)));
    e.set_label(duel, count as i64);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(material_check_eff) = e.get_label_object(duel) else {
        return Ok(false);
    };
    let label = material_check_eff.get_label(duel);
    if chk == 0 {
        return Ok(label > 0);
    }
    material_check_eff.set_label(duel, 0);
    duel.set_target_param(label as u32);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, Some(e.get_handler(duel).into()), 1, tp, 800);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_ONFIELD) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POSSESSED]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let max_count: u32 = duel.get_chain_info::<u32>(0, CHAININFO_TARGET_PARAM).min(4);
    let mut remaining_count = max_count;
    let necro_spfilter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    loop {
        let b1 = c.is_relate_to_effect(duel, e) && c.is_faceup(duel);
        let b2 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
        let b3 = duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None);
        let b4 = duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(&necro_spfilter, tp, LOCATION_GRAVE, 0, 1, None);
        if !(b1 || b2 || b3 || b4) {
            return Ok(());
        }
        let b5 = remaining_count < max_count;
        let options = [
            (b1, aux::stringid(ID, 1)),
            (b2, aux::stringid(ID, 2)),
            (b3, aux::stringid(ID, 3)),
            (b4, aux::stringid(ID, 4)),
            (b5, aux::stringid(ID, 5)),
        ];
        let op = duel.select_effect(tp, &options)?;
        remaining_count -= 1;
        match op {
            Some(1) => {
                c.update_attack(duel, 800, None, None, None);
            }
            Some(2) => {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
                let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
                if !g.is_empty() {
                    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                    duel.confirm_cards(1 - tp, g)?;
                }
            }
            Some(3) => {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
                let g = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
                if !g.is_empty() {
                    duel.hint_selection(g);
                    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                }
            }
            Some(4) => {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let g = duel.select_matching_card(tp, &necro_spfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
                if !g.is_empty() {
                    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
                }
            }
            Some(5) => return Ok(()),
            _ => {}
        }
        if remaining_count == 0 {
            return Ok(());
        }
    }
}
