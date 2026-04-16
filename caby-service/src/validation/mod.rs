use std::fmt;

use email_address::EmailAddress;

use crate::error::Result;

pub mod prefabs;

#[derive(Debug)]
pub struct ValidationError(pub String);

#[derive(Debug)]
pub struct ValidationErrors(Vec<ValidationError>);

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, err) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", err.0)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

pub type RuleGroup<T> = Vec<Rule<T>>;

pub fn exec_group<T: Copy>(group: &RuleGroup<T>, value: T) -> ValidationErrors {
    ValidationErrors(group.iter().filter_map(|rule| rule(value)).collect())
}

pub type RuleStack<T> = Vec<RuleGroup<T>>;

pub fn exec_stack<T: Copy>(stack: &RuleStack<T>, value: T) -> Option<ValidationErrors> {
    for group in stack {
        let res = exec_group(group, value);
        if res.0.len() > 0 {
            return Some(res);
        }
    }
    return None;
}

pub fn exec_stack_optional<T: Copy>(
    stack: &RuleStack<T>,
    value: Option<T>,
) -> Option<ValidationErrors> {
    value.and_then(|v| exec_stack(stack, v))
}

pub type Rule<T> = Box<dyn Fn(T) -> Option<ValidationError>>;

pub fn max<'a>(value: usize) -> Rule<&'a str> {
    Box::new(move |s: &str| {
        if s.len() > value {
            return Some(ValidationError(format!("must be less than {} characters", value)).into());
        }
        None
    })
}

pub fn min<'a>(value: usize) -> Rule<&'a str> {
    Box::new(move |s: &str| {
        if s.len() < value {
            return Some(ValidationError(format!("must be at least {} characters", value)).into());
        }
        None
    })
}

pub fn len<'a>(value: usize) -> Rule<&'a str> {
    Box::new(move |s: &str| {
        if s.len() < value {
            return Some(ValidationError(format!("must be exactly {} characters", value)).into());
        }
        None
    })
}

pub fn no_whitespace<'a>() -> Rule<&'a str> {
    Box::new(|s: &str| {
        if s.contains(char::is_whitespace) {
            return Some(ValidationError("must not contain whitespace".to_string()).into());
        }
        None
    })
}

pub fn trimmed<'a>() -> Rule<&'a str> {
    Box::new(|s: &str| {
        if s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace) {
            return Some(
                ValidationError("must not contain leading or trailing spaces".to_string()).into(),
            );
        }
        None
    })
}

pub fn no_slash<'a>() -> Rule<&'a str> {
    Box::new(|s: &str| {
        if s.contains('/') {
            return Some(ValidationError("must not contain slashes".to_string()).into());
        }
        None
    })
}

pub fn email<'a>() -> Rule<&'a str> {
    Box::new(|s: &str| {
        if !EmailAddress::is_valid(s) {
            return Some(ValidationError("invalid email address".to_string()).into());
        }
        None
    })
}
