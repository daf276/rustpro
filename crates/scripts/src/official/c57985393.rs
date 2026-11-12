// Broomy
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 57985393;

// Two copies in the hand with Key Mace (Level 1 Normal Monster) as the other monster.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (1929294, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e0.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_level_below(duel, 8))));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rm_chk: bool, sp_chk: bool) -> bool {
    c.is_monster(duel)
        && !c.is_public(duel)
        && ((sp_chk && c.is_able_to_remove(duel)) || (rm_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let rm_chk = c.is_able_to_remove(duel);
    let sp_chk = c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let filter = |c, duel: &Duel| spcostfilter(c, duel, e, tp, rm_chk, sp_chk);
    if chk == 0 {
        return Ok((rm_chk || sp_chk)
            && !c.is_public(duel)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let mut g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, tp, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, g: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && g.is_exists(duel, CardId::is_able_to_remove, 1, Some(c.into()))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 && g.len() == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp, g), 1, 1, None)?;
        if sg.len() == 1 {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            duel.remove(g.difference(sg), POS_FACEUP, REASON_EFFECT)?;
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !c.is_type(duel, TYPE_SYNCHRO) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| !c.is_type(duel, TYPE_SYNCHRO));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}
