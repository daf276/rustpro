// Gagaga Magician - Gagaga Magic
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 12908094;

pub const LISTED_NAMES: &[u32] = &[ID];

// Gagaga Ganbara Knight in the hand and Graveyard to Special Summon with it, and Gagaga Cowboy and Zubaba General (Rank 4) to Xyz Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (9491461, HAND),
    (9491461, GRAVE),
    (12014404, EXTRA),
    (12014404, EXTRA),
    (31563350, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(12014404)];

const SETCODES: &[u16] = &[SET_ZUBABA, SET_GAGAGA, SET_GOGOGO, SET_DODODO];
const XYZ_SETCODES: &[u16] = &[
    SET_ZUBABA,
    SET_GAGAGA,
    SET_GOGOGO,
    SET_DODODO,
    SET_DJINN,
    SET_HEROIC_CHAMPION,
    SET_NUMBER,
    SET_ZW,
    SET_ZS,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn samexyzlevel(duel: &mut Duel, handler: CardId, mc: CardId, lv: u32) -> EffectId {
    let e1 = duel.create_effect(handler);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_XYZ_LEVEL);
    e1.set_value(duel, lv as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    mc.register_effect_full(duel, e1, true);
    e1
}

fn handgyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    if c.is_code(duel, ID) {
        return false;
    }
    if !(c.is_set_card(duel, SETCODES) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    duel.check_mut(|duel| {
        let e1 = samexyzlevel(duel, c, c, mc.get_level(duel));
        let e2 = samexyzlevel(duel, c, mc, c.get_level(duel));
        let mg = CardSet::from_iter([mc, c]);
        let res = duel.is_existing_matching_card(|xc, duel| extraspfilter(xc, duel, mg), tp, LOCATION_EXTRA, 0, 1, None);
        e1.reset(duel);
        e2.reset(duel);
        res
    })
}

fn extraspfilter(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_set_card(duel, XYZ_SETCODES) && c.is_xyz_monster(duel) && c.is_xyz_summonable_full(duel, None, Some(mg), 2, 2)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, None) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |sc, duel| handgyspfilter(sc, duel, e, tp, c),
                tp,
                LOCATION_HAND | LOCATION_GRAVE,
                0,
                1,
                Some(c.into()),
            ));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        3,
        tp,
        (LOCATION_HAND | LOCATION_GRAVE | LOCATION_EXTRA) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && duel.get_mzone_count(tp, None) >= 2) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|sc, duel| handgyspfilter(sc, duel, e, tp, c));
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    let g = CardSet::from_iter([c, sc]);
    if duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 2 {
        let e1 = samexyzlevel(duel, c, c, sc.get_level(duel));
        let e2 = samexyzlevel(duel, c, sc, c.get_level(duel));
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let xyz = duel
            .select_matching_card(tp, |xc, duel| extraspfilter(xc, duel, g), tp, LOCATION_EXTRA, 0, 1, 1, None)?
            .get_first();
        if let Some(xyz) = xyz {
            duel.xyz_summon_full(tp, xyz, Some(g), None, 2, 2)?;
        } else {
            e1.reset(duel);
            e2.reset(duel);
        }
    }
    Ok(())
}
