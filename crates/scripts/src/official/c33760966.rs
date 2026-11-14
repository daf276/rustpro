// Dracotail Arthalion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 33760966;

pub const LISTED_SERIES: &[u16] = &[SET_DRACOTAIL];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::Filter(|c, duel, _, _, _| c.is_location(duel, LOCATION_HAND));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
        c.is_set_card_full(duel, &[SET_DRACOTAIL], Some(fc), sumtype, tp)
    })];
}

// In the Extra Deck, with Dracotail Faimena, Lukias and Urgula in hand to Fusion Summon it, and a copy in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (75003700, HAND),
    (1498449, HAND),
    (70871153, MZONE),
    (ID, GRAVE),
    (70871153, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(
        duel,
        Value::MaterialCheck(|duel, e, c| {
            let count = c
                .get_material(duel)
                .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
            e.set_label(duel, count as i64);
        }),
    );
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_fusion_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e0| e0.get_label(duel) > 0)
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
    register_revive::<ID>(duel, c, |duel, _| !duel.is_phase(PHASE_DAMAGE));
}

// The Dracotail Fusion Monsters' effects to Special Summon themselves from the Graveyard when monsters are sent there at
// the same time: e2 and the effects that track it.
pub fn register_revive<const ID: u32>(duel: &mut Duel, c: CardId, condition: fn(&Duel, &Args) -> bool) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_CUSTOM + ID);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, condition);
    e3.set_operation(duel, regop::<ID>);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).reset_flag_effect(duel, ID + 1);
        Ok(())
    });
    c.register_effect(duel, e4);
    for code in [EVENT_CHAIN_SOLVED, EVENT_BREAK_EFFECT] {
        let e = e4.clone_effect(duel);
        e.set_code(duel, code);
        c.register_effect(duel, e);
    }
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, locations, locations, 1, None));
    }
    let ct = a.e.get_label_object(duel).map_or(0, |e0| e0.get_label(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, locations, locations, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn regop<const ID: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = a.eg.unwrap_or_default().filter_count(duel, CardId::is_monster, None);
    if ct == 0 || a.eg.is_some_and(|eg| eg.contains(c)) {
        return Ok(());
    }
    if ct >= 2 || c.has_flag_effect(duel, ID + 1) {
        duel.raise_single_event(c, EVENT_CUSTOM + ID, a.re, a.r, a.rp, a.ep, a.ev)?;
    }
    if duel.get_chain_info::<Option<EffectId>>(0, CHAININFO_TRIGGERING_EFFECT).is_some() {
        c.register_flag_effect(duel, ID + 1, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1);
    }
    Ok(())
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
