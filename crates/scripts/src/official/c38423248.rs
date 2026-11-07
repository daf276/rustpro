// Invoked Magistus Omega
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 38423248;
const ALEISTER_THE_INVOKER: u32 = 86120751;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(ALEISTER_THE_INVOKER),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_EXTRA, Some(fc), sumtype, tp)),
    ];
}

pub const LISTED_NAMES: &[u32] = &[ALEISTER_THE_INVOKER];

// Fusion Summoned by Invocation from Aleister the Invoker in hand and Invoked Sorath on the field, with another Sorath in
// the Graveyard and a monster of the opponent's to banish, and a Level 4 Spellcaster in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (74063034, HAND),
    (86120751, HAND),
    (43989315, MZONE),
    (43989315, GRAVE),
    (43989315, MZONE | OPPONENT),
    (15025844, DECK),
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
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

// function(e,c) return c==sc end, with sc as the label object group.
pub fn equip_limit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c))
}

fn rmfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_can_be_effect_target(duel, e) && c.is_able_to_remove(duel) && (c.is_type(duel, TYPE_EXTRA) || c.is_location(duel, LOCATION_MZONE))
}

fn rmrescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None) == 1,
        false,
    )
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| rmfilter(c, duel, e),
        tp,
        LOCATION_GRAVE | LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rmrescon));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rmrescon, tp, HINTMSG_REMOVE)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(sg), 2, tp, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel
        .get_target_cards(a.e)
        .unwrap_or_default()
        .filter(duel, CardId::is_monster, None);
    if !tg.is_empty() {
        duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_fusion_summoned(duel) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT) && c.is_previous_location(duel, LOCATION_MZONE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[4]) && c.is_race(duel, RACE_SPELLCASTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && c.is_relate_to_effect(duel, e)
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        if duel.equip(tp, c, sc)? {
            let e0 = duel.create_effect(c);
            e0.set_type(duel, EFFECT_TYPE_SINGLE);
            e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e0.set_code(duel, EFFECT_EQUIP_LIMIT);
            e0.set_value(duel, Value::Card(equip_limit));
            e0.set_label_object_group(duel, Some(sc.into()));
            e0.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect(duel, e0);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_EQUIP);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, 1000);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
