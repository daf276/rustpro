// Cyber Jar
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 34124316;

// Face-down on the field for both players, to be flipped by an attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET,
    );
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, LOCATION_MZONE);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 0, PLAYER_ALL, LOCATION_DECK as i32);
    Ok(false)
}

fn spchk(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK | POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn summon(duel: &mut Duel, mut g: CardSet, p: u8, tograve: &mut CardSet, ft: i32) -> Script<()> {
    if g.is_empty() {
        return Ok(());
    }
    if ft == 0 {
        tograve.merge(g);
        return Ok(());
    }
    if ft < g.len() as i32 {
        duel.hint(HINT_SELECTMSG, p, HINTMSG_SPSUMMON);
        let newg = g.select(duel, p, ft as u8, ft as u8, None)?;
        tograve.merge(g.difference(newg));
        g = newg;
    }
    for tc in g.iter() {
        duel.special_summon_step(tc, 0, p, p, false, false, POS_FACEUP_ATTACK | POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    if duel.destroy(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.break_effect()?;
    let p = duel.get_turn_player();
    let (mut summonable1, mut nonsummonable1) = duel.get_decktop_group(p, 5).split(duel, |c, duel| spchk(c, duel, e, p), None);
    let (mut summonable2, nonsummonable2) = duel
        .get_decktop_group(1 - p, 5)
        .split(duel, |c, duel| spchk(c, duel, e, 1 - p), None);
    let ft1 = duel.get_location_count(p, LOCATION_MZONE);
    if ft1 > 1 && duel.is_player_affected_by_effect(p, CARD_BLUEEYES_SPIRIT).is_some() && summonable1.len() > 1 {
        nonsummonable1.merge(summonable1);
        summonable1 = CardSet::default();
    }
    let mut nonsummonable2 = nonsummonable2;
    let ft2 = duel.get_location_count(1 - p, LOCATION_MZONE);
    if ft2 > 1 && duel.is_player_affected_by_effect(1 - p, CARD_BLUEEYES_SPIRIT).is_some() && summonable2.len() > 1 {
        nonsummonable2.merge(summonable2);
        summonable2 = CardSet::default();
    }
    nonsummonable1.merge(nonsummonable2);
    let (tohand, mut tograve) = nonsummonable1.split(duel, CardId::is_able_to_hand, None);
    duel.disable_shuffle_check(true);
    duel.confirm_decktop(p, 5)?;
    summon(duel, summonable1, p, &mut tograve, ft1)?;
    duel.confirm_decktop(1 - p, 5)?;
    summon(duel, summonable2, 1 - p, &mut tograve, ft2)?;
    duel.special_summon_complete()?;
    if !tohand.is_empty() {
        duel.send_to_hand(tohand, PLAYER_NONE, REASON_EFFECT)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_hand(1 - tp)?;
    }
    if !tograve.is_empty() {
        duel.send_to_grave(tograve, REASON_EFFECT)?;
    }
    let fg = duel.get_matching_group(CardId::is_facedown, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.shuffle_set_card(fg)
}
