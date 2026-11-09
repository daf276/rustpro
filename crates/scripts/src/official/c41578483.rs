// Millennium-Eyes Restrict
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, OPPONENT};

const ID: u32 = 41578483;
const RELINQUISHED: u32 = 64631466;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(RELINQUISHED),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_EFFECT, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Relinquished and Sangan, against the opponent's Ash Blossom & Joyous
// Spring and Ghost Ogre & Snow Rabbit in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (RELINQUISHED, HAND),
    (26202165, HAND),
    (14558127, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
    (59438930, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel))
    });
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_equip_group(duel).filter_count(duel, eqgfilter, None) > 0
    });
    e2.set_value(duel, Value::Card(|duel, _, c| equipped_text_sum(duel, c, CardId::get_text_attack)));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e3.set_value(duel, Value::Card(|duel, _, c| equipped_text_sum(duel, c, CardId::get_text_defense)));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_ATTACK);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_target_card(duel, distg);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_DISABLE);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e6.set_code(duel, EVENT_CHAIN_SOLVING);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_condition(duel, discon);
    e6.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e6);
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && c.is_able_to_change_controler(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && eqfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(eqfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, eqfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && eqfilter(tc, duel)
        && tc.is_controler(duel, 1 - tp)
    {
        e.get_handler(duel)
            .equip_by_effect_and_limit_register(duel, e, tp, tc, Some(ID), None)?;
    }
    Ok(())
}

fn eqgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_flag_effect(duel, ID)
}

fn equipped_text_sum(duel: &Duel, c: CardId, stat: fn(CardId, &Duel) -> i32) -> i32 {
    c.get_equip_group(duel)
        .filter(duel, eqgfilter, None)
        .filter(duel, |c, duel| stat(c, duel) > 0, None)
        .get_sum(duel, |c, duel| i64::from(stat(c, duel))) as i32
}

fn distg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let eqg = e.get_handler(duel).get_equip_group(duel).filter(duel, eqgfilter, None);
    eqg.is_exists(
        duel,
        |ec, duel| ec.is_original_code_rule(duel, c.get_original_code_rule(duel)),
        1,
        None,
    )
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let eqg = a.e.get_handler(duel).get_equip_group(duel).filter(duel, eqgfilter, None);
    a.re.is_some_and(|re| {
        re.is_monster_effect(duel)
            && eqg.is_exists(
                duel,
                |ec, duel| ec.is_original_code_rule(duel, re.get_handler(duel).get_original_code_rule(duel)),
                1,
                None,
            )
    })
}
