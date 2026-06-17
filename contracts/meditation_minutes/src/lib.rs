#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol};

/// `MeditationMinutes` is an on-chain meditation tracker for the
/// Stellar / Soroban network. Users log individual sessions
/// (minutes + technique + date) and accumulate a verifiable
/// lifetime total. Once a lifetime-minute threshold is reached,
/// the user can self-claim a permanent achievement badge that
/// anyone can read on-chain.
#[contract]
pub struct MeditationMinutes;

#[contractimpl]
impl MeditationMinutes {
    /// Log a meditation session for `user`. The user must authorize
    /// the call via `require_auth`. `minutes` must be in the range
    /// `(0, 1440]` (no zero-length sessions, no sessions longer than
    /// a single day). `technique` is a short `Symbol` identifying
    /// the practice (e.g. `"mindfulness"`, `"breath"`, `"loving_k"`).
    ///
    /// Returns the user's new total lifetime minutes.
    pub fn log_session(
        env: Env,
        user: Address,
        minutes: u32,
        technique: Symbol,
    ) -> u32 {
        // Authentication: only the user themselves may add to
        // their own log.
        user.require_auth();

        // Input validation.
        if minutes == 0 {
            panic!("minutes must be greater than zero");
        }
        if minutes > 1440 {
            panic!("session cannot exceed 24 hours (1440 minutes)");
        }

        // 1. Update the lifetime-minute total.
        let mut lifetime: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&"lifetime")
            .unwrap_or_else(|| Map::new(&env));
        let updated = lifetime
            .get(user.clone())
            .unwrap_or(0)
            .checked_add(minutes)
            .expect("lifetime minutes overflow");
        lifetime.set(user.clone(), updated);
        env.storage().instance().set(&"lifetime", &lifetime);

        // 2. Increment the session counter for this user.
        let mut counts: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&"session_count")
            .unwrap_or_else(|| Map::new(&env));
        let count = counts
            .get(user.clone())
            .unwrap_or(0)
            .checked_add(1)
            .expect("session count overflow");
        counts.set(user.clone(), count);
        env.storage().instance().set(&"session_count", &counts);

        // 3. Persist the individual session record under a composite
        //    key so it can be read later. Value is a tuple
        //    (minutes, technique, ledger_timestamp).
        let session_key = (Symbol::new(&env, "session"), user, count);
        env.storage().instance().set(
            &session_key,
            &(minutes, technique, env.ledger().timestamp()),
        );

        updated
    }

    /// Return the total lifetime minutes meditated by `user`.
    /// Returns `0` for users who have never logged a session.
    pub fn lifetime_minutes(env: Env, user: Address) -> u32 {
        let lifetime: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&"lifetime")
            .unwrap_or_else(|| Map::new(&env));
        lifetime.get(user).unwrap_or(0)
    }

    /// Return the number of meditation sessions logged by `user`.
    /// Returns `0` for users who have never logged a session.
    pub fn session_count(env: Env, user: Address) -> u32 {
        let counts: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&"session_count")
            .unwrap_or_else(|| Map::new(&env));
        counts.get(user).unwrap_or(0)
    }

    /// Claim a milestone achievement badge for `user` at the given
    /// `threshold` of lifetime minutes. The user must authorize the
    /// call and must have already reached the threshold.
    ///
    /// Returns `true` if the badge was newly claimed in this call,
    /// and `false` if the user had already claimed it previously
    /// (the call is idempotent and never panics on a duplicate
    /// claim). Panics if the user has not yet reached `threshold`.
    pub fn claim_badge(env: Env, user: Address, threshold: u32) -> bool {
        user.require_auth();

        if threshold == 0 {
            panic!("threshold must be greater than zero");
        }

        let current = Self::lifetime_minutes(env.clone(), user.clone());
        if current < threshold {
            panic!("lifetime minutes below threshold");
        }

        let badge_key = (Symbol::new(&env, "badge"), user, threshold);
        if env
            .storage()
            .instance()
            .get::<_, bool>(&badge_key)
            .unwrap_or(false)
        {
            // Already claimed — idempotent no-op.
            return false;
        }

        env.storage().instance().set(&badge_key, &true);
        true
    }

    /// Return `true` if `user` has already claimed a badge at the
    /// given `threshold` of lifetime minutes, `false` otherwise.
    /// This is a read-only helper and does not require auth.
    pub fn has_badge(env: Env, user: Address, threshold: u32) -> bool {
        let badge_key = (Symbol::new(&env, "badge"), user, threshold);
        env.storage()
            .instance()
            .get::<_, bool>(&badge_key)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_log_and_query() {
        let env = Env::default();
        env.mock_all_auths();
        let user = Address::generate(&env);
        let technique = Symbol::new(&env, "mindfulness");

        let total = MeditationMinutes::log_session(env.clone(), user.clone(), 10, technique);
        assert_eq!(total, 10);
        assert_eq!(MeditationMinutes::lifetime_minutes(env.clone(), user.clone()), 10);
        assert_eq!(MeditationMinutes::session_count(env.clone(), user.clone()), 1);

        let total = MeditationMinutes::log_session(env, user, 15, technique);
        assert_eq!(total, 25);
    }

    #[test]
    fn test_badge_claim_idempotent() {
        let env = Env::default();
        env.mock_all_auths();
        let user = Address::generate(&env);
        let technique = Symbol::new(&env, "breath");

        MeditationMinutes::log_session(env.clone(), user.clone(), 100, technique);

        // First claim is new.
        let first = MeditationMinutes::claim_badge(env.clone(), user.clone(), 100);
        assert!(first);
        // Second claim is idempotent.
        let second = MeditationMinutes::claim_badge(env.clone(), user.clone(), 100);
        assert!(!second);
        // has_badge reflects both calls.
        assert!(MeditationMinutes::has_badge(env, user, 100));
    }

    #[test]
    #[should_panic]
    fn test_claim_below_threshold_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let user = Address::generate(&env);
        let technique = Symbol::new(&env, "yoga");

        MeditationMinutes::log_session(env.clone(), user.clone(), 5, technique);
        // User only has 5 minutes; trying to claim a 100-minute
        // badge must panic.
        MeditationMinutes::claim_badge(env, user, 100);
    }

    #[test]
    #[should_panic]
    fn test_zero_minutes_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let user = Address::generate(&env);
        let technique = Symbol::new(&env, "mantra");
        MeditationMinutes::log_session(env, user, 0, technique);
    }
}
