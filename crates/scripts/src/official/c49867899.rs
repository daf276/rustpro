// Fiendsmith's Sequence
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 49867899;

// With Invoked Purgatrio to Fusion Summon from Aleister and Darkfire Soldier #1 in the GY, and Evil HERO Adusted Gold (LIGHT Fiend) to equip to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (12307878, EXTRA),
    (86120751, GRAVE),
    (5388481, GRAVE),
    (13650422, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(12307878),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| matfilter(c, duel, lc, sumtype, tp), 1, None)
    }
}

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_FIEND)
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = aux::necro_valley_filter(|c, duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_deck));
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None)
    }

    fn extraop(duel: &mut Duel, e: EffectId, tc: CardId, tp: u8, sg: &mut CardSet) -> Script<()> {
        Fusion::shuffle_material(duel, e, tc, tp, sg)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, a.tp, LOCATION_GRAVE as i32);
        Ok(false)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(lc), sumtype, tp) && c.is_race_full(duel, RACE_FIEND, Some(lc), sumtype, tp)
}

pub fn eqfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_link_monster(duel) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_FIEND) && c.is_faceup(duel)
}

pub fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && eqfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(eqfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, eqfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, tp, 0);
    if c.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, tp, 0);
    }
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_controler(duel, tp)
        && duel.equip(tp, c, tc)?
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
        e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    } else if c.is_location(duel, LOCATION_MZONE) {
        duel.send_to_grave_full(c, REASON_RULE, PLAYER_NONE, PLAYER_NONE)?;
    }
    Ok(())
}
