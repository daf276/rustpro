// Infinitrack Fortress Megaclops
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 69073023;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_XYZ, Some(lc), sumtype, tp)
    }
}

// On the field with Infinitrack River Stormer (Xyz) to keep and in the Graveyard to revive, and the opponent's Ash Blossom to take as material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (24701066, MZONE),
    (24701066, MZONE),
    (24701066, MZONE),
    (24701066, GRAVE),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(
        duel,
        Value::Effect(|duel, _, te| te.is_monster_effect(duel) && !te.is_active_type(duel, TYPE_XYZ)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_type(duel, TYPE_XYZ))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn mtfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_type(duel, TYPE_TOKEN) && c.is_able_to_change_controler(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_target(mtfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g1 = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g1, g1.map_or(0, |g| g.len() as u8), 0, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    duel.select_target(tp, mtfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg: CardSet = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let tc = tg.filter(duel, |c, duel| spfilter(c, duel, e, tp), None).first();
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && tc.is_location(duel, LOCATION_GRAVE)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? != 0
    {
        let oc = tg.filter(duel, mtfilter, Some(tc.into())).first();
        if let Some(oc) = oc
            && oc.is_controler(duel, 1 - tp)
            && oc.is_relate_to_effect(duel, e)
            && !oc.is_immune_to_effect(duel, e)
        {
            oc.cancel_to_grave(duel);
            duel.overlay_full(tc, oc, true)?;
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::ChangeDamage(|_, _, _, val, _, _, _| val / 2));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, e.get_handler(duel), 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}
