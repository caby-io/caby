use std::sync::LazyLock;

use crate::validation::*;

// todo: figure out how to make const/statics from these

pub fn username_validation<'a>() -> RuleStack<&'a str> {
    return vec![{ vec![max(255), min(2), no_whitespace(), no_slash()] }];
}

pub fn email_validation<'a>() -> RuleStack<&'a str> {
    return vec![{ vec![email()] }];
}

pub fn activation_token_validation<'a>() -> RuleStack<&'a str> {
    return vec![{ vec![len(64), no_whitespace()] }];
}
