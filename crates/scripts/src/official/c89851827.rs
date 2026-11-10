// Secreterion Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 89851827;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

// Baby Dragon, Flame Manipulator and Polymerization to Fusion Summon it from the Extra Deck; Petit Dragon and Mystical Elf
// in the Graveyard for the Special Summon and Deck return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (24094653, HAND),
    (88819587, HAND),
    (34460851, HAND),
    (75356564, GRAVE),
    (15025844, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| -100 * duel.get_field_group_count(e.get_handler_player(duel), LOCATION_REMOVED, 0) as i32),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e2.set_value(duel, Value::EffectPlayer(actlimval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, sptdtg);
    e3.set_operation(duel, sptdop);
    c.register_effect(duel, e3);
}

fn actlimval(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel)
        && rc.is_special_summoned(duel)
        && rc.is_faceup(duel)
        && rc.is_race(duel, RACE_DRAGON | RACE_SPELLCASTER)
        && rc.is_location(duel, LOCATION_MZONE)
}

fn sptdfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON | RACE_SPELLCASTER)
        && (c.is_can_be_special_summoned(duel, e, 0, tp, false, false) || c.is_able_to_deck(duel))
}

fn spchk(c: CardId, duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (sg.len() == 1 || sg.is_exists(duel, CardId::is_able_to_deck, 1, Some(c.into())))
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.is_race(duel, RACE_DRAGON), 1, None)
            && sg.is_exists(duel, |c, duel| c.is_race(duel, RACE_SPELLCASTER), 1, None)
            && sg.is_exists(duel, |c, duel| spchk(c, duel, e, tp, sg), 1, None),
        false,
    )
}

fn sptdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| sptdfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && g.len() >= 2
            && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, e, tp)));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, e, tp), tp, HINTMSG_TARGET)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), 1, tp, 0);
    Ok(false)
}

fn sptdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = tg.filter_select(duel, tp, |c, duel| spchk(c, duel, e, tp, tg), 1, 1, None)?;
    if !sg.is_empty() && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0 && tg.len() == 2 {
        let dg = tg.difference(sg);
        duel.hint_selection(dg);
        duel.send_to_deck(dg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    }
    Ok(())
}
