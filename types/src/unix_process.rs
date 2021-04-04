use serde::{Deserialize, Serialize};
use std::{
    char::ParseCharError,
    convert::Infallible,
    num::{ParseFloatError, ParseIntError},
};
use thiserror::Error;

macro_rules! generate_from_str {
    (struct $name:ident ( $err_type:ident , $err_kind_type:ty ) {
        $(($n:literal) $field:ident % $ty:ident)*
    }) => {
        #[derive(Debug, Serialize, Deserialize)]
        pub struct $name {
            $(pub $field: scanf_types:: $ty,)*
        }

        impl std::str::FromStr for $name {
            type Err = $err_type;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut entries = s.split(' ');
                Ok( $name {
                    $($field : entries
                        .next()
                        .ok_or(< $err_kind_type >::UnexpectedEnd)
                        .and_then(|v| v.parse::<scanf_types:: $ty>().map_err(From::from))
                        .map_err(|kind| $err_type {kind, n: $n})?
                        ,)*
                })

            }
        }
    };
}

#[allow(non_camel_case_types, dead_code)]
mod scanf_types {
    pub type d = i32;
    pub type u = u32;
    pub type ld = i64;
    pub type lu = u64;
    pub type lld = i128;
    pub type llu = u128;
    pub type s = String;
    pub type c = char;
}

generate_from_str! {
    struct Process (ProcessParseError, ProcessParseErrorKind) {
        (1) pid  %d
        (2) comm  %s
        (3) state  %c
        (4) ppid  %d
        (5) pgrp  %d
        (6) session  %d
        (7) tty_nr  %d
        (8) tpgid  %d
        (9) flags  %u
        (10) minflt  %lu
        (11) cminflt  %lu
        (12) majflt  %lu
        (13) cmajflt  %lu
        (14) utime  %lu
        (15) stime  %lu
        (16) cutime  %ld
        (17) cstime  %ld
        (18) priority  %ld
        (19) nice  %ld
        (20) num_threads  %ld
        (21) itrealvalue  %ld
        (22) starttime  %llu
        (23) vsize  %lu
        (24) rss  %ld
        (25) rsslim  %lu
        (26) startcode  %lu
        (27) endcode  %lu
        (28) startstack  %lu
        (29) kstkesp  %lu
        (30) kstkeip  %lu
        (31) signal  %lu
        (32) blocked  %lu
        (33) sigignore  %lu
        (34) sigcatch  %lu
        (35) wchan  %lu
        (36) nswap  %lu
        (37) cnswap  %lu
        (38) exit_signal  %d
        (39) processor  %d
        (40) rt_priority  %u
        (41) policy  %u
        (42) delayacct_blkio_ticks  %llu
        (43) guest_time  %lu
        (44) cguest_time  %ld
        (45) start_data  %lu
        (46) end_data  %lu
        (47) start_brk  %lu
        (48) arg_start  %lu
        (49) arg_end  %lu
        (50) env_start  %lu
        (51) env_end  %lu
        // (52) exit_code  %d
    }
}

#[derive(Error, Debug)]
pub enum ProcessParseErrorKind {
    #[error("float: {0}")]
    Float(#[from] ParseFloatError),
    #[error("int: {0}")]
    Int(#[from] ParseIntError),
    #[error("char: {0}")]
    Char(#[from] ParseCharError),
    #[error("unexpected end")]
    UnexpectedEnd,
    #[error(transparent)]
    __Infallible(#[from] Infallible),
}
#[derive(Error, Debug)]
#[error("error while parsing `/proc/[pid]/stat`'s {kind} at {n}")]
pub struct ProcessParseError {
    kind: ProcessParseErrorKind,
    n: u32,
}
