// Grapha, Dragon Overlord of Dark World
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, OPPONENT};

const ID: u32 = 39552584;
const GRAPHA_LORD: u32 = 34230233;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(GRAPHA_LORD),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp)),
    ];
}

pub const LISTED_NAMES: &[u32] = &[GRAPHA_LORD];

// In the Extra Deck, Fusion Summoned by Polymerization with Grapha, Dragon Lord and Renge, and a second Grapha in the
// Graveyard to return. The opponent's Raigeki gets its effect changed, or destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (GRAPHA_LORD, HAND),
    (60606759, HAND),
    (GRAPHA_LORD, GRAVE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, chcon);
    e1.set_target(duel, chtg);
    e1.set_operation(duel, chop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn chcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    a.rp == 1 - a.tp
        && (re.is_monster_effect(duel)
            || ((rc.is_normal_spell(duel) || rc.is_normal_trap(duel)) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)))
}

fn chtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, LOCATION_HAND, 0) > 0);
    }
    Ok(false)
}

fn chop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, repop);
    Ok(())
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.get_field_group(tp, 0, LOCATION_HAND).is_empty() {
        duel.discard_hand(1 - tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.rp == 1 - tp && c.is_fusion_summoned(duel) && c.is_previous_location(duel, LOCATION_MZONE) && c.is_previous_controler(duel, tp)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_code(duel, GRAPHA_LORD) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && (duel.get_field_group_count(tp, LOCATION_HAND, 0) > 0 || duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0)
    {
        let turn_player = duel.get_turn_player();
        duel.break_effect()?;
        duel.discard_hand(turn_player, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
        duel.discard_hand(1 - turn_player, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}
