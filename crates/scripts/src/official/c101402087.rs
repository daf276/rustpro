// Skyborne Blue Cularsaw
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 101402087;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(lv8matfilter),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_WARRIOR, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Tri-Horned Dragon and Dark Blade, Warriors in the Graveyard to equip;
// on the field with Megamorph to send, and the opponent's Raigeki and Mystical Space Typhoon to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (22046459, HAND),
    (22046459, HAND),
    (39111158, MZONE),
    (39111158, MZONE),
    (11321183, MZONE),
    (11321183, MZONE),
    (11321183, GRAVE),
    (12580477, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp
            && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
            && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
            && duel.is_chain_negatable(a.ev as u8)
    });
    e2.set_cost(duel, negcost);
    e2.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
        Ok(false)
    });
    e2.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
}

const LOCATIONS_GRAVE_REMOVED: u32 = LOCATION_GRAVE | LOCATION_REMOVED;

fn lv8matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_level_above(duel, 8) && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp)
}

fn eqfilter(c: CardId, duel: &Duel, opp: u8, tp: u8) -> bool {
    (c.is_race(duel, RACE_WARRIOR) || (c.is_monster(duel) && c.is_controler(duel, opp)))
        && c.is_faceup(duel)
        && c.check_unique_on_field(duel, tp)
        && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| eqfilter(c, duel, 1 - tp, tp),
                tp,
                LOCATIONS_GRAVE_REMOVED,
                LOCATIONS_GRAVE_REMOVED,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, PLAYER_EITHER, LOCATIONS_GRAVE_REMOVED as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let filter = aux::necro_valley_filter(|c, duel| eqfilter(c, duel, 1 - tp, tp));
        let g = duel.select_matching_card(tp, filter, tp, LOCATIONS_GRAVE_REMOVED, LOCATIONS_GRAVE_REMOVED, 1, 1, None)?;
        let Some(sc) = g.get_first() else { return Ok(()) };
        duel.hint_selection(sc);
        equipop(duel, c, e, tp, sc)?;
    }
    Ok(())
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, Some(true))? {
        let e1a = duel.create_effect(tc);
        e1a.set_type(duel, EFFECT_TYPE_EQUIP);
        e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1a.set_value(duel, 500);
        e1a.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1a);
        let e1b = e1a.clone_effect(duel);
        e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
        tc.register_effect(duel, e1b);
    }
    Ok(())
}

fn negcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_equip_card(duel) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, negcostfilter, LOCATION_SZONE)
}
