// Utopic Onomatopoeia
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 8512558;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_ZUBABA, SET_GAGAGA, SET_GOGOGO, SET_DODODO];
// Zubababa Knight, Gagaga Ganbara Knight, Gogogo Goblindbergh and Dodododo Warrior in the hand to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (62006866, HAND),
    (9491461, HAND),
    (35886170, HAND),
    (62880279, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

const SETCODES: [u32; 4] = [SET_ZUBABA as u32, SET_GAGAGA as u32, SET_GOGOGO as u32, SET_DODODO as u32];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.add_setcodes_rule(duel, ID, true, &[SET_ZUBABA]);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ZUBABA, SET_GAGAGA, SET_GOGOGO, SET_DODODO])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn rescon(duel: &Duel, tp: u8, sg: CardSet) -> (bool, bool) {
    let checkfunc = aux::property_table_filter(|c, duel| c.get_set_card(duel).into_iter().map(u32::from).collect(), &SETCODES);
    let stop = !aux::chkf_mmz(tp, sg.len() as i32)(duel, sg, sg, None).0 || !aux::check_different_property(duel, sg, &checkfunc);
    (true, stop)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        let check = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| rescon(duel, tp, sg);
        return Ok(ft > 0 && aux::select_unselect_group_check(duel, g, 1, ft as usize, &check));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE).min(4);
    if !g.is_empty() && ft > 0 {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let check = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| rescon(duel, tp, sg);
        let sg = aux::select_unselect_group_full(duel, g, 1, ft as usize, &check, tp, HINTMSG_SPSUMMON, Some(&check), false)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_XYZ));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_XYZ));
    Ok(())
}
