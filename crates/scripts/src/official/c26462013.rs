// Primite Dragon Nether Berzelius
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 26462013;
pub const MATERIAL_SETCODE: &[u16] = &[SET_PRIMITE];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::Filter(|c, duel, _, _, _| c.is_type(duel, TYPE_NORMAL));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(|c, duel, _, _, _| c.is_set_card(duel, &[SET_PRIMITE]))];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Primite Dragon Ether Beryl and Mystical Elf (Normal Monster), with
// more Mystical Elf in the Graveyard and Deck, against the opponent's Jurrac Megalo and Raigeki to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (63198739, HAND),
    (15025844, HAND),
    (15025844, HAND),
    (15025844, GRAVE),
    (15025844, GRAVE),
    (15025844, DECK),
    (93170499, MZONE | OPPONENT),
    (93170499, MZONE | OPPONENT),
    (93170499, MZONE | OPPONENT),
    (93170499, MZONE | OPPONENT),
    (93170499, HAND | OPPONENT),
    (93170499, HAND | OPPONENT),
    (15025844, MZONE),
    (12580477, HAND | OPPONENT),
];
// The negation is a continuous effect with no chain link and no hint a check can count.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_operation(duel, |duel, a| {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let count = c
        .get_material(duel)
        .filter_count(duel, |c, duel| c.is_type(duel, TYPE_NORMAL), None);
    c.update_attack(
        duel,
        count as i32 * 1000,
        Some(RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD)),
        None,
        None,
    );
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) = a.re else { return false };
    if !re.is_monster_effect(duel) || a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    let normal_ct = duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_NORMAL)),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        None,
    ) as u32;
    if normal_ct == 0 {
        return false;
    }
    let trig_ctrl: u8 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_CONTROLER);
    let trig_loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    let trig_lv: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LEVEL);
    let trig_rk: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_RANK);
    if !(trig_ctrl == 1 - tp && trig_loc == LOCATION_MZONE) {
        return false;
    }
    let trig_lk = re.get_handler(duel).get_link(duel);
    (trig_lv > 0 && trig_lv <= normal_ct) || (trig_rk > 0 && trig_rk <= normal_ct) || (trig_lk > 0 && trig_lk <= normal_ct)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
