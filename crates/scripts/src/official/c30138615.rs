// Nightmare-Eyes Restrict
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 30138615;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(matfilter); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with a Fiend and a Spellcaster, with the opponent's monsters to equip and attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (24094653, HAND),
    (12057781, HAND),
    (27125110, HAND),
    (12057781, MZONE),
    (27125110, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_EQUIP);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, eqtg);
    e1a.set_operation(duel, eqop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1b.set_code(duel, EVENT_PHASE + PHASE_BATTLE);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    c.register_effect(duel, e1b);
    aux::add_er_equip_limit(duel, c, None, e1a);
    aux::add_er_equip_limit(duel, c, None, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_equip_group(duel)
            .is_exists(duel, aux::faceup_filter(|c, duel| c.has_flag_effect(duel, ID)), 1, None)
    });
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| {
            let filter = |c: CardId, duel: &Duel| c.has_flag_effect(duel, ID) && c.is_faceup(duel) && c.get_text_attack(duel) > 0;
            c.get_equip_group(duel)
                .filter(duel, filter, None)
                .get_sum(duel, |c, duel| i64::from(c.get_text_attack(duel))) as i32
        }),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, e, c| {
        let handler = e.get_handler(duel);
        c == handler || Some(c) == handler.get_battle_target(duel)
    });
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 0, LOCATION_MZONE);
    e4.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e4);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    c.is_race_full(duel, RACE_FIEND | RACE_ILLUSION | RACE_SPELLCASTER, Some(fc), sumtype, tp)
        && sg.is_none_or(|sg| {
            sg.filter_count(duel, aux::TRUE, Some(c.into())) == 0
                || !sg.is_exists(
                    duel,
                    |mc, duel| mc.is_race_full(duel, c.get_race(duel), Some(fc), sumtype, tp),
                    1,
                    Some(c.into()),
                )
        })
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let sc = duel.select_matching_card(tp, CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if let Some(sc) = sc.get_first() {
        duel.hint_selection(sc);
        c.equip_by_effect_and_limit_register(duel, e, tp, sc, Some(ID), None)?;
    }
    Ok(())
}
