// Divine Scorpion Beast of Serket
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 23804920;

pub const LISTED_SERIES: &[u16] = &[SET_SERKET];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_SERKET], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_attack_below(duel, 2500)),
    ];
}

// Fusion Summoned by Polymerization with two Mystical Beasts of Serket, with the opponent's Sangan to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24094653, HAND),
    (89194033, HAND),
    (89194033, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_condition(duel, rmcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_EXTRA_ATTACK);
    e3.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_level_above(duel, 10),
            0,
            LOCATION_REMOVED,
            LOCATION_REMOVED,
            1,
            None,
        )
    });
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|bc| bc.is_controler(duel, 1 - a.tp))
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_monster(duel) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let atk = f64::from(tc.get_base_attack(duel)) / 2.0;
    if tc.is_relate_to_effect(duel, a.e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && c.is_relate_to_effect(duel, a.e)
        && c.is_faceup(duel)
    {
        c.update_attack(duel, atk.round() as i32, None, None, None);
    }
    Ok(())
}
