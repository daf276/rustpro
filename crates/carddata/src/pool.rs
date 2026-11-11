use std::collections::HashMap;

use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};

use crate::cards::{Card, CardDatabase, TYPE_EXTRA, TYPE_NORMAL, TYPE_PENDULUM, TYPE_TOKEN};

const OT_TCG: u32 = 0x2;
const MAIN_SIZE: usize = 40;
const EXTRA_MAX: usize = 15;
const COPIES_MAX: u8 = 3;

// lflist.conf: the first `!name` line opens the current list; entries are `<id> <limit> --name`.
pub fn banlist_limits(banlist: &str) -> HashMap<u32, u8> {
    banlist
        .lines()
        .skip_while(|line| !line.starts_with('!'))
        .skip(1)
        .take_while(|line| !line.starts_with('!'))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub main: Vec<u32>,
    pub extra: Vec<u32>,
}

impl Deck {
    // Lines `main`, `extra` and `side` of comma-separated codes, alternate arts mapped to their original; no side deck.
    pub fn parse(text: &str, cards: &CardDatabase) -> Self {
        let section = |name: &str| {
            let codes = text.lines().find_map(|line| line.strip_prefix(name)?.strip_prefix(' '));
            original(codes.unwrap_or_default().split(','), cards)
        };
        Self {
            main: section("main"),
            extra: section("extra"),
        }
    }

    // EDOPro's .ydk: a code per line after `#main` and `#extra`, the side deck after `!side`, other `#` lines comments.
    pub fn parse_ydk(text: &str, cards: &CardDatabase) -> Self {
        let section = |name: &str| {
            let lines = text.lines().map(str::trim).skip_while(|&line| line != name).skip(1);
            original(lines.take_while(|line| !line.starts_with(['#', '!'])), cards)
        };
        Self {
            main: section("#main"),
            extra: section("#extra"),
        }
    }

    pub fn shuffled(&self, rng: &mut impl Rng) -> Self {
        let mut deck = self.clone();
        deck.main.shuffle(rng);
        deck
    }
}

fn original<'a>(codes: impl Iterator<Item = &'a str>, cards: &CardDatabase) -> Vec<u32> {
    codes
        .filter_map(|code| code.trim().parse().ok())
        .map(|code| match cards.get(code) {
            Some(card) if card.is_alternate_art() => card.alias,
            _ => code,
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub main: Vec<u32>,
    pub extra: Vec<u32>,
    limits: HashMap<u32, u8>,
}

impl Pool {
    pub fn advanced(cards: &CardDatabase, limits: HashMap<u32, u8>) -> Self {
        let (extra, main): (Vec<&Card>, Vec<&Card>) = cards
            .cards()
            .filter(|card| card.ot & OT_TCG != 0 && card.card_type & TYPE_TOKEN == 0 && !card.is_alternate_art())
            .filter(|card| limits.get(&card.code) != Some(&0))
            .partition(|card| card.card_type & TYPE_EXTRA != 0);
        let codes = |cards: Vec<&Card>| cards.iter().map(|card| card.code).collect();
        Self {
            main: codes(main),
            extra: codes(extra),
            limits,
        }
    }

    // Normal Monsters have no scripts; Normal Pendulums do.
    pub fn retain_normal_monsters(&mut self, cards: &CardDatabase) {
        let normal = |&code: &u32| {
            cards
                .get(code)
                .is_some_and(|card| card.card_type & (TYPE_NORMAL | TYPE_PENDULUM) == TYPE_NORMAL)
        };
        self.main.retain(normal);
        self.extra.retain(normal);
    }

    pub fn limit(&self, code: u32) -> u8 {
        self.limits.get(&code).copied().unwrap_or(COPIES_MAX)
    }

    pub fn random_deck(&self, rng: &mut impl Rng) -> Deck {
        let mut main = self.random_cards(&self.main, MAIN_SIZE, rng);
        main.shuffle(rng);
        let extra_size = rng.random_range(0..=EXTRA_MAX);
        Deck {
            main,
            extra: self.random_cards(&self.extra, extra_size, rng),
        }
    }

    // A random deck whose main deck holds zero to two copies of each of `cards`, filled up with the pool's cards.
    pub fn random_deck_with(&self, cards: &[u32], rng: &mut impl Rng) -> Deck {
        let mut deck = self.random_deck(rng);
        if cards.is_empty() {
            return deck;
        }
        let mut main: Vec<u32> = cards
            .iter()
            .flat_map(|&code| std::iter::repeat_n(code, rng.random_range(0..=2)))
            .collect();
        main.extend(deck.main.iter().take(MAIN_SIZE.saturating_sub(main.len())));
        main.shuffle(rng);
        deck.main = main;
        deck
    }

    // Distinct cards, each with a random legal number of copies.
    fn random_cards(&self, candidates: &[u32], size: usize, rng: &mut impl Rng) -> Vec<u32> {
        let mut cards = Vec::with_capacity(size);
        for &code in candidates.choose_multiple(rng, size) {
            if cards.len() == size {
                break;
            }
            let copies = rng.random_range(1..=usize::from(self.limit(code)).min(size - cards.len()));
            cards.extend(std::iter::repeat_n(code, copies));
        }
        cards
    }
}

// Random decks from the pool with zero to two copies of each of `effect_cards`, or `deck` shuffled if given.
pub struct Decks {
    pub pool: Pool,
    pub effect_cards: Vec<u32>,
    pub deck: Option<Deck>,
}

impl Decks {
    pub fn deal(&self, rng: &mut impl Rng) -> Deck {
        match &self.deck {
            Some(deck) => deck.shuffled(rng),
            None => self.pool.random_deck_with(&self.effect_cards, rng),
        }
    }
}

#[cfg(feature = "cdb")]
pub fn load_advanced(vendor: &std::path::Path) -> Result<(CardDatabase, Pool), crate::CardsError> {
    let cards = CardDatabase::load(&vendor.join("BabelCDB"))?;
    let banlist = std::fs::read_to_string(vendor.join("LFLists/0TCG.lflist.conf")).map_err(crate::CardsError::Banlist)?;
    let pool = Pool::advanced(&cards, banlist_limits(&banlist));
    Ok((cards, pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_the_main_and_extra_deck_of_a_ydk() {
        let ydk = "#created by someone
#main
10
20
#extra
30
!side
40
";
        let deck = Deck::parse_ydk(ydk, &CardDatabase::default());
        assert_eq!((deck.main, deck.extra), (vec![10, 20], vec![30]));
    }
}
