#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod game {
    use erc20::Erc20Ref;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    #[ink(storage)]
    pub struct Game {
        players: Vec<AccountId>,
        score_distance_time: Mapping<AccountId, (u64, u64, u64)>, // score, distance, time
        winners: Vec<AccountId>,
        game_active: bool,
        game_end_time: Timestamp,
        admin: AccountId,
        token: Erc20Ref,
    }
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        GameNotActive,
        NotAdmin,
    }
    #[ink(event)]
    pub struct GameCreated {
        admin: AccountId,
        game_end_time: Timestamp,
        game_start_time: Timestamp,
    }
    #[ink(event)]
    pub struct GameEnded {
        winners: Vec<AccountId>,
        game_end_time: Timestamp,
    }
    #[ink(event)]
    pub struct ScoreSubmitted {
        winner: AccountId,
        score_time_distance: (u64, u64, u64),
    }
    #[ink::trait_definition]
    pub trait BaseGame {
        #[ink(message)]
        fn submit_score(&mut self, score: u64, distance: u64, time: u64) -> Result<(), Error>;
        #[ink(message)]
        fn game_active(&self) -> bool;
        #[ink(message)]
        fn players(&self) -> Vec<AccountId>;
        #[ink(message)]
        fn winners(&self) -> Vec<AccountId>;
        #[ink(message)]
        fn current_admin(&self) -> AccountId;
        #[ink(message)]
        fn change_admin(&mut self, new_admin: AccountId) -> Result<(), Error>;
        #[ink(message)]
        fn game_end_time(&self) -> Timestamp;
        #[ink(message)]
        fn score_distance_time(&self, player: AccountId) -> (u64, u64, u64);
        #[ink(message)]
        fn end_game(&mut self);
    }

    impl Game {
        #[ink(constructor)]
        pub fn new(days: u64, admin: AccountId, token: Hash) -> Self {
            // 86_400_000 is the number of milliseconds in a day
            if let Some(results) = 86_400_000u64.checked_mul(days) {
                let token_contract = Erc20Ref::new(100_000_000_000)
                    .code_hash(token)
                    .endowment(0)
                    .salt_bytes([0xDE, 0xAD, 0xBE, 0xEF])
                    .ref_time_limit(500_000_000)
                    .proof_size_limit(200_000)
                    .storage_deposit_limit(100_000_000_000)
                    .instantiate();
                Self::env().emit_event(GameCreated {
                    admin,
                    game_end_time: Self::env()
                        .block_timestamp()
                        .checked_add(results)
                        .unwrap_or_default(),
                    game_start_time: Self::env().block_timestamp(),
                });
                Self {
                    game_active: true,
                    game_end_time: Self::env()
                        .block_timestamp()
                        .checked_add(results)
                        .unwrap_or_default(),
                    players: Vec::new(),
                    score_distance_time: Mapping::default(),
                    winners: Vec::new(),
                    admin,
                    token: token_contract,
                }
            } else {
                panic!("Game duration too long");
            }
        }
    }

    impl BaseGame for Game {
        #[ink(message)]
        fn game_active(&self) -> bool {
            self.game_active
        }
        #[ink(message)]
        fn players(&self) -> Vec<AccountId> {
            self.players.clone()
        }
        #[ink(message)]
        fn winners(&self) -> Vec<AccountId> {
            self.winners.clone()
        }
        #[ink(message)]
        fn current_admin(&self) -> AccountId {
            self.admin
        }
        #[ink(message)]
        fn change_admin(&mut self, new_admin: AccountId) -> Result<(), Error> {
            if self.env().caller() == self.admin {
                self.admin = new_admin;
                return Ok(());
            }
            Err(Error::NotAdmin)
        }
        #[ink(message)]
        fn submit_score(&mut self, score: u64, distance: u64, time: u64) -> Result<(), Error> {
            // Ideally we would want to ensure that these value are submitted from within the game
            // As such would require an additional field which contains a signed message from the game which can be
            // decoded by the contract to ensure that the values are valid and signed by the current admin of the game
            // But since this is for testing purposes we will just assume that the values are valid
            if self.game_active() {
                self.score_distance_time
                    .insert(self.env().caller(), &(score, distance, time));
                if !self.players.contains(&self.env().caller()) {
                    self.players.push(self.env().caller());
                }
                self.env().emit_event(ScoreSubmitted {
                    winner: self.env().caller(),
                    score_time_distance: (score, distance, time),
                });
                return Ok(());
            }
            Err(Error::GameNotActive)
        }
        #[ink(message)]
        fn end_game(&mut self) {
            if self.env().block_timestamp() >= self.game_end_time {
                self.game_active = false;
                self.env().emit_event(GameEnded {
                    winners: self.winners.clone(),
                    game_end_time: self.game_end_time,
                });
                let mut players = self.players();
                players.sort_by(|a, b| {
                    self.score_distance_time(*a)
                        .0
                        .cmp(&self.score_distance_time(*b).0)
                });
                players.sort_by(|a, b| {
                    self.score_distance_time(*a)
                        .2
                        .cmp(&self.score_distance_time(*b).2)
                });
                //@dev We dont relaly care too much for the distance so we will just sort by score and time
            }
        }
        #[ink(message)]
        fn game_end_time(&self) -> Timestamp {
            self.game_end_time
        }
        #[ink(message)]
        fn score_distance_time(&self, player: AccountId) -> (u64, u64, u64) {
            self.score_distance_time.get(player).unwrap_or_default()
        }
        fn top_three(players: Vec<AccountId>) -> Vec<AccountId> {
            if players.len() < 3 {
                return players;
            }
            players.into_iter().take(3).collect()
        }
        fn update_scores(&mut self) {
            let mut players = self.players();
            players.sort_by(|a, b| {
                self.score_distance_time(*a)
                    .0
                    .cmp(&self.score_distance_time(*b).0)
            });
            players.sort_by(|a, b| {
                self.score_distance_time(*a)
                    .2
                    .cmp(&self.score_distance_time(*b).2)
            });
            //@dev We dont relaly care too much for the distance so we will just sort by score and time
            self.winners = Self::top_three(players);
        }
    }
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
        #[ink_e2e::test]
        async fn e2e_multi_contract_caller<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = Erc20Ref::new(100_000_000_000);
            // given
            let game_hash = client
                .upload("erc20", &ink_e2e::alice())
                .submit()
                .await
                .expect("uploading `game` failed")
                .code_hash;
            let contract = client
                .instantiate("erc20", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("uploading `game` failed");
            let game = Game::new(100, AccountId::from([0x1; 32]), game_hash);
            assert_eq!(game.game_active(), true);
            assert_eq!(game.game_end_time(), 86_400_000 * 100);
            Ok(())
        }
    }
}
