// Ports of playerop.cpp: write the prompt at step 0, then validate the response at every later step. DUEL_SIMPLE_AI
// is unsupported.
use crate::card::{CardId, LocInfo};
use crate::constants::*;
use crate::duel::Duel;
use crate::list::List;
use crate::processor::{
    AnnounceAttribute, AnnounceRace, SelectBattleCmd, SelectCard, SelectChain, SelectEffectYesNo, SelectIdleCmd, SelectOption, SelectPlace,
    SelectPosition, SelectSum, SelectTributeP, SelectUnselectCard, SelectYesNo, SortCard,
};

impl Duel {
    pub(crate) fn process_select_battle_cmd(&mut self, arg: &mut SelectBattleCmd) -> bool {
        if arg.step == 0 {
            self.sort_select_chains();
            let entries = self.chain_entries();
            let mut message = self.trace.message(MSG_SELECT_BATTLECMD);
            message.u8(arg.playerid).u32(self.core.select_chains.len() as u32);
            for &(code, location, description, mode) in &entries {
                message
                    .u32(code)
                    .u8(location.controler)
                    .u8(location.location)
                    .u32(location.sequence)
                    .u64(description)
                    .u8(mode);
            }
            message.u32(self.core.attackable_cards.len() as u32);
            for &card in &self.core.attackable_cards {
                let card = &self.cards[card];
                let current = card.current;
                message
                    .u32(card.data.code)
                    .u8(current.controler)
                    .u8(current.location)
                    .u8(current.sequence as u8)
                    .u8(card.direct_attackable);
            }
            message.u8(u8::from(self.core.to_m2)).u8(u8::from(self.core.to_ep));
            return false;
        }
        let response = self.returns.at_i32(0);
        let (kind, index) = ((response & 0xffff) as u32, (response >> 16) as u32 as usize);
        if kind > 3
            || (kind == 0 && index >= self.core.select_chains.len())
            || (kind == 1 && index >= self.core.attackable_cards.len())
            || (kind == 2 && !self.core.to_m2)
            || (kind == 3 && !self.core.to_ep)
        {
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    pub(crate) fn process_select_idle_cmd(&mut self, arg: &mut SelectIdleCmd) -> bool {
        let playerid = arg.playerid;
        let can_shuffle = self.infos.can_shuffle && self.player[playerid].list_hand.len() > 1;
        let to_bp = self.infos.phase == PHASE_MAIN1 && self.core.to_bp;
        if arg.step == 0 {
            self.sort_select_chains();
            let entries = self.chain_entries();
            let core = &self.core;
            let mut message = self.trace.message(MSG_SELECT_IDLECMD);
            message.u8(playerid);
            for (index, list) in [
                &core.summonable_cards,
                &core.spsummonable_cards,
                &core.repositionable_cards,
                &core.msetable_cards,
                &core.ssetable_cards,
            ]
            .into_iter()
            .enumerate()
            {
                message.u32(list.len() as u32);
                for &card in list {
                    let card = &self.cards[card];
                    message.u32(card.data.code).u8(card.current.controler).u8(card.current.location);
                    // Repositionable cards are written with a one-byte sequence.
                    if index == 2 {
                        message.u8(card.current.sequence as u8);
                    } else {
                        message.u32(card.current.sequence);
                    }
                }
            }
            message.u32(core.select_chains.len() as u32);
            for &(code, location, description, mode) in &entries {
                message
                    .u32(code)
                    .u8(location.controler)
                    .u8(location.location)
                    .u32(location.sequence)
                    .u64(description)
                    .u8(mode);
            }
            message.u8(u8::from(to_bp)).u8(u8::from(self.core.to_ep)).u8(u8::from(can_shuffle));
            return false;
        }
        let response = self.returns.at_i32(0);
        let (kind, index) = ((response & 0xffff) as u32, (response >> 16) as u32 as usize);
        let core = &self.core;
        if kind > 8
            || (kind == 0 && index >= core.summonable_cards.len())
            || (kind == 1 && index >= core.spsummonable_cards.len())
            || (kind == 2 && index >= core.repositionable_cards.len())
            || (kind == 3 && index >= core.msetable_cards.len())
            || (kind == 4 && index >= core.ssetable_cards.len())
            || (kind == 5 && index >= core.select_chains.len())
            || (kind == 6 && !to_bp)
            || (kind == 7 && !core.to_ep)
            || (kind == 8 && !can_shuffle)
        {
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    pub(crate) fn process_select_chain(&mut self, arg: &mut SelectChain) -> bool {
        let playerid = arg.playerid;
        if arg.step == 0 {
            self.returns.set_i32(0, -1);
            let hint_timing = self.core.hint_timing;
            self.sort_select_chains();
            let entries = self.chain_entries();
            let mut message = self.trace.message(MSG_SELECT_CHAIN);
            message.u8(playerid).u8(arg.spe_count).u8(u8::from(arg.forced));
            message
                .u32(hint_timing[playerid])
                .u32(hint_timing[1 - playerid])
                .u32(self.core.select_chains.len() as u32);
            for &(code, location, description, mode) in &entries {
                message.u32(code).loc(location).u64(description).u8(mode);
            }
            return false;
        }
        let response = self.returns.at_i32(0);
        if !arg.forced && response == -1 {
            return true;
        }
        if usize::try_from(response).is_ok_and(|index| index < self.core.select_chains.len()) {
            return true;
        }
        self.trace.message(MSG_RETRY);
        false
    }

    // The candidates in the order of their effects' ids (chain::chain_operation_sort), as the prompts list them.
    fn sort_select_chains(&mut self) {
        let mut chains = std::mem::take(&mut self.core.select_chains);
        self.sort_chains(&mut chains);
        self.core.select_chains = chains;
    }

    // Each candidate's handler code and location, effect description and client mode, as the prompts write them.
    fn chain_entries(&self) -> List<(u32, LocInfo, u64, u8), 8> {
        self.core
            .select_chains
            .iter()
            .map(|chain| {
                let effect = &self.effects[chain.triggering_effect];
                let card = self.get_handler(chain.triggering_effect);
                (
                    self.cards[card].data.code,
                    self.cards.get_info_location(card),
                    effect.description,
                    effect.get_client_mode(),
                )
            })
            .collect()
    }

    pub(crate) fn process_select_effect_yes_no(&mut self, arg: &mut SelectEffectYesNo) -> bool {
        if arg.step == 0 {
            let (code, location) = (self.cards[arg.card].data.code, self.cards.get_info_location(arg.card));
            self.trace
                .message(MSG_SELECT_EFFECTYN)
                .u8(arg.playerid)
                .u32(code)
                .loc(location)
                .u64(arg.description);
            self.returns.set_i32(0, -1);
            return false;
        }
        if !matches!(self.returns.at_i32(0), 0 | 1) {
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    pub(crate) fn process_select_yes_no(&mut self, arg: &mut SelectYesNo) -> bool {
        if arg.step == 0 {
            self.trace.message(MSG_SELECT_YESNO).u8(arg.playerid).u64(arg.description);
            self.returns.set_i32(0, -1);
            return false;
        }
        if !matches!(self.returns.at_i32(0), 0 | 1) {
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    // Skipped without options; a single option is still asked.
    pub(crate) fn process_select_option(&mut self, arg: &mut SelectOption) -> bool {
        if arg.step == 0 {
            self.returns.set_i32(0, -1);
            if self.core.select_options.is_empty() {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(arg.playerid).u64(0);
                return true;
            }
            let mut message = self.trace.message(MSG_SELECT_OPTION);
            message.u8(arg.playerid).u8(self.core.select_options.len() as u8);
            for &option in &self.core.select_options {
                message.u64(option);
            }
            return false;
        }
        if !usize::try_from(self.returns.at_i32(0)).is_ok_and(|index| index < self.core.select_options.len()) {
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    pub(crate) fn process_announce_race(&mut self, arg: &mut AnnounceRace) -> bool {
        if arg.step == 0 {
            if arg.count == 0 {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(arg.playerid).u64(0);
                return true;
            }
            self.trace
                .message(MSG_ANNOUNCE_RACE)
                .u8(arg.playerid)
                .u8(arg.count)
                .u64(arg.available);
            self.returns.clear();
            return false;
        }
        let selected = self.returns.at_u64(0);
        if selected & !arg.available != 0 || selected.count_ones() != u32::from(arg.count) {
            self.trace.message(MSG_RETRY);
            return false;
        }
        self.trace.message(MSG_HINT).u8(HINT_RACE).u8(arg.playerid).u64(selected);
        true
    }

    pub(crate) fn process_announce_attribute(&mut self, arg: &mut AnnounceAttribute) -> bool {
        if arg.step == 0 {
            if arg.count == 0 {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(arg.playerid).u64(0);
                return true;
            }
            self.trace
                .message(MSG_ANNOUNCE_ATTRIB)
                .u8(arg.playerid)
                .u8(arg.count)
                .u32(arg.available);
            self.returns.clear();
            return false;
        }
        let selected = self.returns.at_u32(0);
        if selected & !arg.available != 0 || selected.count_ones() != u32::from(arg.count) {
            self.trace.message(MSG_RETRY);
            return false;
        }
        self.trace
            .message(MSG_HINT)
            .u8(HINT_ATTRIB)
            .u8(arg.playerid)
            .u64(u64::from(selected));
        true
    }

    // The answer is the new position of each card, one byte per card.
    pub(crate) fn process_sort_card(&mut self, arg: &mut SortCard) -> bool {
        let playerid = arg.playerid;
        if arg.step == 0 {
            self.returns.clear();
            if self.core.select_cards.is_empty() {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(0);
                return true;
            }
            let cards: Vec<_> = self
                .core
                .select_cards
                .iter()
                .map(|&card| (self.cards[card].data.code, self.cards[card].current))
                .collect();
            let mut message = self.trace.message(if arg.is_chain { MSG_SORT_CHAIN } else { MSG_SORT_CARD });
            message.u8(playerid).u32(cards.len() as u32);
            for (code, current) in cards {
                message
                    .u32(code)
                    .u8(current.controler)
                    .u32(current.location.into())
                    .u32(current.sequence);
            }
            return false;
        }
        if self.returns.at_u8(0) == u8::MAX {
            return true;
        }
        let count = self.core.select_cards.len();
        let mut seen = vec![false; count];
        for index in 0..count {
            let position = usize::from(self.returns.at_u8(index));
            match seen.get_mut(position) {
                Some(slot) if !*slot => *slot = true,
                _ => {
                    self.trace.message(MSG_RETRY);
                    return false;
                }
            }
        }
        true
    }

    pub(crate) fn process_select_card(&mut self, arg: &mut SelectCard) -> bool {
        let playerid = arg.playerid;
        if arg.step == 0 {
            self.return_cards.clear();
            self.returns.clear();
            if arg.max == 0 || self.core.select_cards.is_empty() {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(0);
                return true;
            }
            arg.max = arg.max.min(self.core.select_cards.len() as u8);
            arg.min = arg.min.min(arg.max);
            let mut cards = std::mem::take(&mut self.core.select_cards);
            cards.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
            self.core.select_cards = cards;
            let mut message = self.trace.message(MSG_SELECT_CARD);
            message.u8(playerid).u8(u8::from(arg.cancelable || arg.min == 0));
            message
                .u32(u32::from(arg.min))
                .u32(u32::from(arg.max))
                .u32(self.core.select_cards.len() as u32);
            for &card in &self.core.select_cards {
                message.u32(self.cards[card].data.code).loc(self.cards.get_info_location(card));
            }
            return false;
        }
        if !self.parse_response_cards(arg.cancelable || arg.min == 0) {
            self.return_cards.clear();
            self.trace.message(MSG_RETRY);
            return false;
        }
        if self.return_cards.canceled {
            return true;
        }
        let count = self.return_cards.list.len();
        if count < usize::from(arg.min) || count > usize::from(arg.max) {
            self.return_cards.clear();
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    pub(crate) fn process_select_sum(&mut self, arg: &mut SelectSum) -> bool {
        let playerid = arg.playerid;
        let must = self.core.must_select_cards.get();
        if arg.step == 0 {
            self.return_cards.clear();
            self.returns.clear();
            if self.core.select_cards.is_empty() {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(0);
                return true;
            }
            let mut cards = std::mem::take(&mut self.core.select_cards);
            cards.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
            self.core.select_cards = cards;
            let entries: List<(u32, LocInfo, u32), 16> = must
                .iter()
                .chain(self.core.select_cards.iter().copied())
                .map(|card| (self.cards[card].data.code, self.cards.get_info_location(card), arg.param(card)))
                .collect();
            let mut message = self.trace.message(MSG_SELECT_SUM);
            message.u8(playerid).u8(u8::from(arg.max == 0)).u32((arg.acc & 0xffff) as u32);
            message.u32(arg.min as u32).u32(arg.max.max(arg.min) as u32);
            let (must_entries, select_entries) = entries.split_at_checked(must.len()).unwrap_or_default();
            for list in [must_entries, select_entries] {
                message.u32(list.len() as u32);
                for &(code, location, param) in list {
                    message.u32(code).loc(location).u32(param);
                }
            }
            return false;
        }
        if !self.parse_response_cards(false) {
            self.return_cards.clear();
            self.trace.message(MSG_RETRY);
            return false;
        }
        let params: List<u32, 16> = must
            .iter()
            .chain(self.return_cards.list.iter().copied())
            .map(|card| arg.param(card))
            .collect();
        if !arg.accepts(&params, self.return_cards.list.len()) {
            self.return_cards.clear();
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    pub(crate) fn process_select_unselect_card(&mut self, arg: &mut SelectUnselectCard) -> bool {
        let playerid = arg.playerid;
        if arg.step == 0 {
            self.return_cards.clear();
            self.returns.clear();
            if self.core.select_cards.is_empty() && self.core.unselect_cards.is_empty() {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(0);
                return true;
            }
            let mut cards = std::mem::take(&mut self.core.select_cards);
            cards.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
            self.core.select_cards = cards;
            let mut message = self.trace.message(MSG_SELECT_UNSELECT_CARD);
            message.u8(playerid).u8(u8::from(arg.finishable)).u8(u8::from(arg.cancelable));
            message.u32(u32::from(arg.min)).u32(u32::from(arg.max));
            for list in [&self.core.select_cards, &self.core.unselect_cards] {
                message.u32(list.len() as u32);
                for &card in list {
                    message.u32(self.cards[card].data.code).loc(self.cards.get_info_location(card));
                }
            }
            return false;
        }
        let response = self.returns.at_i32(0);
        if response == -1 && (arg.cancelable || arg.finishable) {
            self.return_cards.canceled = true;
            return true;
        }
        if response == -1 || response == 0 || response > 1 {
            self.return_cards.clear();
            self.trace.message(MSG_RETRY);
            return false;
        }
        let index = self.returns.at_i32(1);
        let selected = usize::try_from(index).ok().and_then(|index| {
            let select_count = self.core.select_cards.len();
            if index < select_count {
                self.core.select_cards.get(index)
            } else {
                self.core.unselect_cards.get(index - select_count)
            }
        });
        let Some(&card) = selected else {
            self.trace.message(MSG_RETRY);
            return false;
        };
        self.return_cards.list.push(card);
        true
    }

    pub(crate) fn process_select_place(&mut self, arg: &mut SelectPlace) -> bool {
        let playerid = arg.playerid;
        if arg.step == 0 {
            if arg.count == 0 {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(0);
                return true;
            }
            self.trace.message(MSG_SELECT_PLACE).u8(playerid).u8(arg.count).u32(arg.flag);
            // The core zeroes the first byte; a response replaces the whole buffer, and `apply` collects zones in it.
            self.returns.clear();
            return false;
        }
        let mut flag = arg.flag;
        for pick in 0..usize::from(arg.count) {
            let (select_player, location, sequence) = (
                self.returns.at_u8(3 * pick),
                self.returns.at_u8(3 * pick + 1),
                self.returns.at_u8(3 * pick + 2),
            );
            let mzone = location == LOCATION_MZONE;
            if select_player > 1 || (!mzone && location != LOCATION_SZONE) || sequence > 7 - u8::from(mzone) {
                self.trace.message(MSG_RETRY);
                return false;
            }
            let to_check = (1u32 << sequence) << (if mzone { 0 } else { 8 } + if select_player == playerid { 0 } else { 16 });
            if to_check & flag != 0 {
                self.trace.message(MSG_RETRY);
                return false;
            }
            flag |= to_check;
        }
        true
    }

    pub(crate) fn process_select_position(&mut self, arg: &mut SelectPosition) -> bool {
        let is_single = |positions: u8| positions.count_ones() == 1 && positions & 0xf != 0;
        if arg.step == 0 {
            if arg.positions == 0 {
                self.returns.set_i32(0, POS_FACEUP_ATTACK as i32);
                return true;
            }
            let positions = arg.positions & 0xf;
            if is_single(positions) {
                self.returns.set_i32(0, i32::from(positions));
                return true;
            }
            self.trace.message(MSG_SELECT_POSITION).u8(arg.playerid).u32(arg.code).u8(positions);
            self.returns.set_i32(0, 0);
            return false;
        }
        let position = self.returns.at_i32(0);
        if !u8::try_from(position).is_ok_and(|position| is_single(position) && position & arg.positions != 0) {
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    pub(crate) fn process_select_tribute_p(&mut self, arg: &mut SelectTributeP) -> bool {
        let playerid = arg.playerid;
        if arg.step == 0 {
            self.returns.clear();
            self.return_cards.clear();
            if arg.max == 0 || self.core.select_cards.is_empty() {
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(0);
                return true;
            }
            let releases = self
                .core
                .select_cards
                .iter()
                .fold(0u8, |total, &card| total.wrapping_add(self.cards[card].release_param as u8));
            arg.max = arg.max.min(5).min(releases);
            arg.min = arg.min.min(arg.max);
            let mut cards = std::mem::take(&mut self.core.select_cards);
            cards.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
            self.core.select_cards = cards;
            let mut message = self.trace.message(MSG_SELECT_TRIBUTE);
            message.u8(playerid).u8(u8::from(arg.cancelable || arg.min == 0));
            message
                .u32(u32::from(arg.min))
                .u32(u32::from(arg.max))
                .u32(self.core.select_cards.len() as u32);
            for &card in &self.core.select_cards {
                let card = &self.cards[card];
                let current = card.current;
                message
                    .u32(card.data.code)
                    .u8(current.controler)
                    .u8(current.location)
                    .u32(current.sequence)
                    .u8(card.release_param as u8);
            }
            return false;
        }
        if !self.parse_response_cards(arg.cancelable || arg.min == 0) {
            self.return_cards.clear();
            self.trace.message(MSG_RETRY);
            return false;
        }
        if self.return_cards.canceled {
            return true;
        }
        let released: u32 = self.return_cards.list.iter().map(|&card| self.cards[card].release_param).sum();
        if self.return_cards.list.len() > usize::from(arg.max) || released < u32::from(arg.min) {
            self.return_cards.clear();
            self.trace.message(MSG_RETRY);
            return false;
        }
        true
    }

    // The core then sorts the cards by address; card ids keep the creation order instead.
    fn parse_response_cards(&mut self, cancelable: bool) -> bool {
        let kind = self.returns.at_i32(0);
        if kind.wrapping_add(1) as u32 > 4 {
            return false;
        }
        if kind == -1 {
            self.return_cards.canceled = cancelable;
            return cancelable;
        }
        let select_cards = &self.core.select_cards;
        let list = &mut self.return_cards.list;
        if kind == 3 {
            list.extend(
                select_cards
                    .iter()
                    .enumerate()
                    .filter(|&(index, _)| self.returns.bit_get(index + 32))
                    .map(|(_, &card)| card),
            );
        } else {
            for index in 0..self.returns.at_u32(1) as usize {
                let selected = match kind {
                    0 => self.returns.at_u32(index + 2) as usize,
                    1 => usize::from(self.returns.at_u16(index + 4)),
                    _ => usize::from(self.returns.at_u8(index + 8)),
                };
                let Some(&card) = select_cards.get(selected) else { return false };
                list.push(card);
            }
        }
        list.sort_unstable();
        !list.windows(2).any(|pair| pair.first() == pair.last())
    }
}

impl SelectSum {
    pub(crate) fn param(&self, card: CardId) -> u32 {
        self.params.iter().find(|&&(other, _)| other == card).map_or(0, |&(_, param)| param)
    }

    // The response check: `params` are the must cards' then the selected cards', `selected` of them. With max 0 the
    // selection must reach acc with every card at its larger half and fall short without the smallest at its smaller.
    pub(crate) fn accepts(&self, params: &[u32], selected: usize) -> bool {
        if self.max != 0 {
            let selected = selected as i32;
            return selected >= self.min && selected <= self.max && select_sum_check1(params, self.acc);
        }
        let (mut sum, mut mx, mut mn) = (0, 0, i32::MAX);
        for &param in params {
            let (o1, o2) = sum_halves(param);
            let ms = if o2 != 0 && o2 < o1 { o2 } else { o1 };
            sum += ms;
            mx += o1.max(o2);
            mn = mn.min(ms);
        }
        mx >= self.acc && sum - mn < self.acc
    }
}

pub(crate) fn sum_halves(param: u32) -> (i32, i32) {
    ((param & 0xffff) as i32, (param >> 16) as i32)
}

fn select_sum_check1(params: &[u32], acc: i32) -> bool {
    let Some((&param, rest)) = params.split_first() else {
        return false;
    };
    if acc == 0 {
        return false;
    }
    let (o1, o2) = sum_halves(param);
    if rest.is_empty() {
        return acc == o1 || acc == o2;
    }
    (acc > o1 && select_sum_check1(rest, acc - o1)) || (o2 > 0 && acc > o2 && select_sum_check1(rest, acc - o2))
}
