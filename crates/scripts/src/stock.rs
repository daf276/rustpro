// Bodies that many card scripts write out identically. A card uses one only where its Lua function is exactly that
// sequence of calls; PORTING.md lists them.
use crate::prelude::*;

pub fn exists_target(
    duel: &mut Duel,
    a: &Args,
    chk: i32,
    filter: impl Fn(CardId, &Duel) -> bool,
    location: u32,
    category: u64,
) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, category, None, 1, a.tp, location as i32);
    Ok(false)
}

pub fn operation_info_target<const CATEGORY: u64, const LOCATION: u32>(
    duel: &mut Duel,
    a: &Args,
    chk: i32,
    _: Option<CardId>,
) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY, None, 1, a.tp, LOCATION as i32);
    Ok(false)
}

pub fn search(duel: &mut Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool, location: u32) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

pub fn send_to_grave(duel: &mut Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool, location: u32) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn special_summon_target(duel: &mut Duel, a: &Args, chk: i32, filter: impl Fn(CardId, &Duel) -> bool, location: u32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(filter, tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

pub fn special_summon(duel: &mut Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool, location: u32, positions: u32) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, positions)?;
    }
    Ok(())
}

pub fn special_summon_self_target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    self_summon_target(duel, a, chk, a.tp)
}

// The Lua names player 0 in the operation info.
pub fn special_summon_self_target_player_0(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    self_summon_target(duel, a, chk, 0)
}

fn self_summon_target(duel: &mut Duel, a: &Args, chk: i32, info_player: u8) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, info_player, 0);
    Ok(false)
}

pub fn special_summon_self<const POSITIONS: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, a.tp, false, false, POSITIONS)?;
    }
    Ok(())
}

pub fn special_summon_first_target<const POSITIONS: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, a.tp, a.tp, false, false, POSITIONS)?;
    }
    Ok(())
}

pub fn destroy_first_target(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn first_target_to_hand(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn self_to_hand(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn draw_target<const COUNT: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, COUNT as usize));
    }
    duel.set_target_player(tp);
    duel.set_target_param(COUNT);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, COUNT as i32);
    Ok(false)
}

pub fn draw(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}

pub fn damage(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}

pub fn negate_destroy_target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

pub fn send_to_grave_cost(duel: &mut Duel, a: &Args, chk: i32, filter: impl Fn(CardId, &Duel) -> bool, location: u32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

pub fn banish_cost(duel: &mut Duel, a: &Args, chk: i32, filter: impl Fn(CardId, &Duel) -> bool, location: u32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}
