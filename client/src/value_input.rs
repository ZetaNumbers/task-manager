#![allow(dead_code)]

use crate::input_util::get_event_value;
use seed::{prelude::*, *};
use std::{fmt::Display, str::FromStr};

pub struct Model<T, E = <T as FromStr>::Err>
where
    T: FromStr<Err = E>,
{
    pub label: String,
    pub value: T,
    pub fallback: String,
    pub error: Option<E>,
}

impl<T> Model<T>
where
    T: FromStr + ToString,
{
    pub fn new(label: String, value: T) -> Self {
        Model {
            label,
            fallback: value.to_string(),
            value,
            error: None,
        }
    }
}

impl<T, E> Model<T, E>
where
    T: FromStr<Err = E>,
    E: Display,
{
    pub fn view(&self) -> Node<Msg> {
        div![
            C!["numeric-input"],
            &self.label,
            input![
                attrs![
                    At::Type => "text",
                    At::Value => self.fallback,
                ],
                self.error.as_ref().map(|error| attrs![
                    At::Class => "input-error",
                    At::Title => error
                ]),
                ev(Ev::Input, |e| Msg(get_event_value(&e)))
            ]
        ]
    }
}

pub struct Msg(String);

impl Msg {
    pub fn update<T, E>(self, model: &mut Model<T, E>)
    where
        T: FromStr<Err = E>,
    {
        match self.0.parse::<T>() {
            Ok(value) => {
                model.value = value;
                model.error = None;
            }
            Err(error) => model.error = Some(error),
        }
        model.fallback = self.0;
    }
}
