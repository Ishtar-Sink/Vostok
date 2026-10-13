//! Confere o próprio reprodutor das fixtures: com os scripts bash originais, todos os casos de cada
//! grupo precisam bater. Ignorados por padrão, porque dependem dos scripts bash, que saem na TASK-11.
//! Uso: `cargo test --test golden -- --ignored`

mod common;

use common::golden::{assert_group_with, Backend};

macro_rules! bash_replay {
    ($($name:ident => $group:literal),* $(,)?) => {
        $(
            #[test]
            #[ignore]
            fn $name() {
                assert_group_with($group, Backend::Bash);
            }
        )*
    };
}

bash_replay! {
    bash_context => "context",
    bash_memory => "memory",
    bash_rules => "rules",
    bash_spec => "spec",
    bash_plan => "plan",
    bash_task => "task",
    bash_commit => "commit",
    bash_skill => "skill",
    bash_finalize => "finalize",
}
