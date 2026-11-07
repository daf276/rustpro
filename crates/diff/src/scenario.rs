use carddata::{Card, CardDatabase, TYPE_EXTRA};
use engine::constants::*;
use engine::{NewCardInfo, PlacedCard, Position};
use rand::Rng;
use rand::seq::IndexedRandom;
use scripts::testing::{DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const OTHER_CARDS_MAX: usize = 3;
const XYZ_MATERIALS_MAX: usize = 3;

// What a scenario places on top of the decks, as a puzzle does: the LP of both players and the cards Debug.AddCard
// places before the duel starts.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub lp: [u32; 2],
    pub cards: Vec<PlacedCard>,
}

impl Scenario {
    // The card under test and its support cards for one player, and up to OTHER_CARDS_MAX of `others` for either, each
    // in a random zone its card type can legally occupy (in the location SUPPORT gives, if any); an Xyz Monster in a
    // Monster Zone gets up to XYZ_MATERIALS_MAX of `others` as materials, owned by either player.
    pub fn generate(card: u32, others: &[u32], database: &CardDatabase, rng: &mut impl Rng) -> Self {
        let player = rng.random_range(0..=1);
        let support = scripts::card(card).map_or(&[][..], |script| script.support);
        let mut placed: Vec<(u32, u8, u16)> = std::iter::once((card, 0))
            .chain(support.iter().copied())
            .map(|(code, location)| match location & OPPONENT {
                0 => (code, player, location),
                _ => (code, 1 - player, location & !OPPONENT),
            })
            .collect();
        for _ in 0..rng.random_range(0..=OTHER_CARDS_MAX) {
            placed.extend(others.choose(rng).map(|&code| (code, rng.random_range(0..=1), 0)));
        }
        let mut used = 0;
        let mut cards = Vec::new();
        for (code, player, location) in placed {
            let Some(card) = database.get(code) else { continue };
            let Some(info) = place(card, player, location, &mut used, rng) else {
                continue;
            };
            cards.push(info);
            if card.card_type & TYPE_XYZ != 0 && info.loc == MZONE.into() {
                for _ in 0..rng.random_range(0..=XYZ_MATERIALS_MAX) {
                    cards.extend(others.choose(rng).map(|&code| NewCardInfo {
                        code,
                        team: rng.random_range(0..=1),
                        ..info
                    }));
                }
            }
        }
        let lp = [0, 1].map(|_| if rng.random() { 8000 } else { rng.random_range(1..=80) * 100 });
        let cards = cards
            .into_iter()
            .map(|info| PlacedCard {
                proc_complete: proc_complete(&info),
                info,
                counters: Vec::new(),
            })
            .collect();
        Self { lp, cards }
    }

    // Where each card is as messages give it (controller, location, sequence), with its code; the Pendulum and Field
    // Zones are Spell & Trap Zones there.
    pub fn places(&self) -> impl Iterator<Item = ((u8, u8, u32), u32)> {
        self.cards.iter().map(|PlacedCard { info, .. }| {
            let (location, sequence) = match info.loc as u16 {
                LOCATION_PZONE => (SZONE, info.seq * 4),
                LOCATION_FZONE => (SZONE, 5),
                location => (location, info.seq),
            };
            ((info.con, location as u8, sequence), info.code)
        })
    }
}

// The Debug calls that set up a position in the oracle, which has none for the turn: the position must start the duel
// (Position::new).
pub fn lua(position: &Position) -> String {
    let players = position.players.iter().enumerate().map(|(player, options)| {
        let (lp, start, draw) = (options.starting_lp, options.starting_draw_count, options.draw_count_per_turn);
        format!("Debug.SetPlayerInfo({player},{lp},{start},{draw})\n")
    });
    let cards = position.cards.iter().map(|card| {
        let NewCardInfo {
            code,
            team,
            con,
            loc,
            seq,
            pos,
            ..
        } = card.info;
        let add = format!("Debug.AddCard({code},{team},{con},{loc},{seq},{pos},{})", card.proc_complete);
        if card.counters.is_empty() {
            return add + "\n";
        }
        let counters = card.counters.iter().map(|&(counter_type, [permanent, negatable])| {
            let negatable_type = counter_type | COUNTER_NEED_ENABLE;
            format!(" Debug.PreAddCounter(c,{counter_type},{permanent}) Debug.PreAddCounter(c,{negatable_type},{negatable})")
        });
        format!("do local c={add}{} end\n", counters.collect::<String>())
    });
    players.chain(cards).collect()
}

// A monster on the field, in the GY, banished or face-up in the Extra Deck got there with its summon procedure complete
// (OCG_DuelNewCard sets it for the Monster Zone alike); only hand and Deck cards lose it.
fn proc_complete(info: &NewCardInfo) -> bool {
    let location = info.loc as u8;
    location & (LOCATION_HAND | LOCATION_DECK) == 0 && (location != LOCATION_EXTRA || info.pos & POS_FACEUP != 0)
}

// A random legal zone and position for the card, in `location` unless that is 0, None when the chosen kind of zone is
// full. `used` holds the occupied zones as the core's used_location bits, player 1's from bit 32.
fn place(card: &Card, player: u8, location: u16, used: &mut u64, rng: &mut impl Rng) -> Option<NewCardInfo> {
    let card_type = card.card_type;
    let extra = card_type & TYPE_EXTRA != 0;
    let mut options = vec![(GRAVE, POS_FACEUP), (REMOVED, POS_FACEUP), (REMOVED, POS_FACEDOWN)];
    if extra {
        options.push((EXTRA, POS_FACEDOWN_DEFENSE));
    } else {
        options.extend([(HAND, POS_FACEDOWN), (DECK, POS_FACEDOWN_DEFENSE)]);
    }
    if card_type & TYPE_MONSTER != 0 {
        let positions: &[u32] = match card_type {
            t if t & TYPE_LINK != 0 => &[POS_FACEUP_ATTACK],
            _ if extra => &[POS_FACEUP_ATTACK, POS_FACEUP_DEFENSE],
            _ => &[POS_FACEUP_ATTACK, POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE],
        };
        options.extend(positions.iter().map(|&position| (MZONE, position)));
        if card_type & TYPE_PENDULUM != 0 {
            options.extend([(EXTRA, POS_FACEUP), (LOCATION_PZONE, POS_FACEUP)]);
        }
    } else if card_type & TYPE_FIELD != 0 {
        options.extend([(LOCATION_FZONE, POS_FACEUP), (LOCATION_FZONE, POS_FACEDOWN)]);
    } else {
        options.push((SZONE, POS_FACEDOWN));
        if card_type & TYPE_CONTINUOUS != 0 {
            options.push((SZONE, POS_FACEUP));
        }
    }
    options.retain(|&(option, _)| location == 0 || option == location);
    let &(location, position) = options.choose(rng)?;
    let bit = |sequence: u32| match location {
        MZONE => Some(sequence),
        SZONE => Some(8 + sequence),
        LOCATION_PZONE => Some(8 + sequence * 4),
        LOCATION_FZONE => Some(13),
        _ => None,
    };
    let sequences = match location {
        MZONE if extra => 0..7,
        MZONE | SZONE => 0..5,
        LOCATION_PZONE | DECK => 0..2,
        _ => 0..1,
    };
    let offset = 32 * u32::from(player);
    let free: Vec<u32> = sequences
        .filter(|&sequence| bit(sequence).is_none_or(|bit| *used & 1 << (offset + bit) == 0))
        .collect();
    let &sequence = free.choose(rng)?;
    *used |= bit(sequence).map_or(0, |bit| 1 << (offset + bit));
    if location == MZONE && sequence >= 5 {
        // The opponent sees the same Extra Monster Zone from the other side.
        *used |= 1 << (32 - offset + 11 - sequence);
    }
    Some(NewCardInfo {
        team: player,
        duelist: 0,
        code: card.code,
        con: player,
        loc: location.into(),
        seq: sequence,
        pos: position,
    })
}
