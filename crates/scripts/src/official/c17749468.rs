// Azamina
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_HINT;

const ID: u32 = 17749468;
const QUEEN_AZAMINA: u32 = 65033975;
const SAINT_AZAMINA: u32 = 85065943;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(QUEEN_AZAMINA), FusionMaterial::Filter(matfilter)];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_AZAMINA];
pub const LISTED_NAMES: &[u32] = &[QUEEN_AZAMINA, SAINT_AZAMINA];

// In the Extra Deck, Special Summoned by Tributing Saint Azamina and an opponent's monster, with Polymerization for a Fusion Summon
// (Queen Azamina and Azamina Mu Rcielago, a Fusion Monster), and opponent's cards to make it lose ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (SAINT_AZAMINA, MZONE),
    (SAINT_AZAMINA, EXTRA),
    (QUEEN_AZAMINA, MZONE),
    (73391962, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
// The ATK loss and the activation lock are continuous effects with no chain link; the loss shows a card hint, which is reported.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::Message(MSG_HINT))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_first_be_fusion_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, selfspcon);
    e0.set_target(duel, selfsptg);
    e0.set_operation(duel, selfspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2a.set_code(duel, EVENT_CHAINING);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2b.set_property(duel, EFFECT_FLAG_DELAY);
    e2b.set_code(duel, EVENT_CHAIN_SOLVED);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, 1));
    e2b.set_operation(duel, atkop);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, 1);
    e3.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && re.get_handler(duel).is_attack(duel, &[0])),
    );
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_FUSION | TYPE_SYNCHRO, Some(fc), sumtype, tp)
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8, fc: CardId) -> bool {
    (c.is_code(duel, SAINT_AZAMINA)
        || c.is_summon_code(duel, Some(fc), MATERIAL_FUSION, tp, &[SAINT_AZAMINA])
        || c.is_controler(duel, 1 - tp))
        && c.is_releasable(duel)
        && c.is_can_be_fusion_material(duel, Some(fc), MATERIAL_FUSION)
        && (c.is_controler(duel, tp) || c.is_faceup(duel))
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (
        duel.get_location_count_from_ex(tp, tp, Some(sg), e.get_handler(duel)) > 0
            && sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 1,
        false,
    )
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let mg = duel.get_matching_group(
        |card, duel| selfspcostfilter(card, duel, tp, c),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    mg.len() >= 2 && aux::select_unselect_group_check(duel, mg, 2, 2, &|duel, sg, _, _| rescon(duel, sg, a.e, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    let mg = duel.get_matching_group(
        |card, duel| selfspcostfilter(card, duel, tp, c),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    let g = aux::select_unselect_group_full(
        duel,
        mg,
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, e, tp),
        tp,
        HINTMSG_RELEASE,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST | REASON_MATERIAL)?;
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if a.rp != 1 - tp {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_CARD, 0, ID.into());
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
