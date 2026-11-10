// Guardian Chimera
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 11321089;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::GroupFilter(ffilter1),
        FusionMaterial::GroupFilter(ffilter2),
        FusionMaterial::GroupFilter(ffilter3),
    ];
    const MATCHECK: Option<MatCheck> = Some(matfilter);
}

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];

// In the Extra Deck, Fusion Summoned by Polymerization with Mystical Elf from the hand and the field and another in the hand, with
// Polymerization in the Graveyard, and monsters of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, GRAVE),
    (15025844, HAND),
    (15025844, HAND),
    (15025844, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// The target protection is a continuous effect, which forms no chain link.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_first_be_fusion_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_label_object(duel, Some(e0));
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_fusion_summoned(duel) && a.re.is_some_and(|re| re.is_spell_effect(duel))
    });
    e1.set_target(duel, drdestg);
    e1.set_operation(duel, drdesop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            |c, duel| c.is_code(duel, CARD_POLYMERIZATION),
            a.e.get_handler_player(duel),
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
    });
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
}

// The Lua calls this with (c, code, fc, sumtype, tp) for its (c, code, fc, tp), so the player is the summon type cut to a byte.
fn fusfilter(c: CardId, duel: &Duel, code: u32, fc: CardId, sumtype: u64) -> bool {
    c.is_summon_code(duel, Some(fc), u64::from(SUMMON_TYPE_FUSION), sumtype as u8, &[code]) && c.is_has_effect(duel, 511002961).is_none()
}

fn unique_code(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    let code = c.get_codes_full(duel, Some(fc), sumtype, tp).first().copied().unwrap_or(0);
    sg.is_none_or(|sg| !sg.is_exists(duel, |sc, duel| fusfilter(sc, duel, code, fc, sumtype), 1, Some(c.into())))
}

fn ffilter1(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    (sumtype == MATERIAL_FUSION || c.is_location(duel, LOCATION_HAND)) && unique_code(c, duel, fc, sumtype, tp, sg)
}

fn ffilter2(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    (sumtype == MATERIAL_FUSION || c.is_location(duel, LOCATION_ONFIELD)) && unique_code(c, duel, fc, sumtype, tp, sg)
}

fn ffilter3(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    unique_code(c, duel, fc, sumtype, tp, sg)
}

fn matfilter(duel: &Duel, c: CardId, fc: CardId, tp: u8, _: bool, sumtype: u64) -> bool {
    if sumtype & u64::from(SUMMON_TYPE_FUSION) > 0 && fc.is_location(duel, LOCATION_EXTRA) {
        return c.is_location(duel, LOCATION_HAND | LOCATION_ONFIELD) && c.is_controler(duel, tp);
    }
    true
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let mg = c.get_material(duel);
    let hc = mg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    let fc = mg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_ONFIELD), None);
    e.set_labels(duel, &[hc as i64, fc as i64]);
}

fn counts(duel: &Duel, e: EffectId) -> (usize, usize) {
    match e.get_label_object(duel).map(|e0| e0.get_labels(duel)) {
        Some(&[hc, fc, ..]) => (hc as usize, fc as usize),
        _ => (0, 0),
    }
}

fn drdestg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let (hc, fc) = counts(duel, a.e);
    let dg = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(hc > 0 && fc > 0 && duel.is_player_can_draw(tp, hc) && dg.len() >= fc);
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, hc as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), fc as u8, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn drdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let (hc, fc) = counts(duel, a.e);
    if duel.draw(p, hc as u32, REASON_EFFECT)? == hc as i32 {
        let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
        if g.len() < fc {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = g.select(duel, tp, fc as u8, fc as u8, None)?;
        if !dg.is_empty() {
            duel.hint_selection(dg);
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
