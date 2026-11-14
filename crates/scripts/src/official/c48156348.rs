// Gladiator Beast Gyzarus
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 48156348;

const BESTIARI: u32 = 41470137;

pub const LISTED_NAMES: &[u32] = &[BESTIARI];
pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GLADIATOR];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(BESTIARI),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_GLADIATOR], Some(fc), sumtype, tp)),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(matfilter, tp, LOCATION_ONFIELD, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, tp: u8, _: CardId) -> Script<()> {
        duel.confirm_cards(1 - tp, g)?;
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

// In the Extra Deck, contact-fused from Bestiari and another Gladiator Beast on the field, and on the field with Gladiator
// Beasts in the Deck to tag in and Sangan to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (BESTIARI, MZONE),
    (25924653, DECK),
    (25924653, DECK),
    (4253484, DECK),
    (4253484, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_cost(duel, Cost::self_to_extra);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_deck_or_extra_as_cost(duel) && (c.is_monster(duel) || c.is_code(duel, BESTIARI))
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    e.get_handler(duel).get_location(duel) != LOCATION_EXTRA
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 2, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let tg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    !c.is_code(duel, BESTIARI) && c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned(duel, e, 120, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, 2, None)?;
    if g.len() == 2 {
        for tc in g {
            duel.special_summon_step(tc, 120, tp, tp, false, false, POS_FACEUP)?;
            let code = tc.get_original_code(duel);
            tc.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 0);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
