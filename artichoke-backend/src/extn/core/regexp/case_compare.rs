//! [`Regexp#===`](https://ruby-doc.org/core-2.6.3/Regexp.html#method-i-3D-3D-3D)

use std::cmp;
use std::mem;

use crate::convert::{Convert, RustBackedValue, TryConvert};
use crate::extn::core::matchdata::MatchData;
use crate::extn::core::regexp::{Backend, Regexp};
use crate::sys;
use crate::value::Value;
use crate::Artichoke;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Error {
    Fatal,
    NoImplicitConversionToString,
}

#[derive(Debug, Clone)]
pub struct Args {
    pub string: Option<String>,
}

impl Args {
    const ARGSPEC: &'static [u8] = b"o\0";

    pub unsafe fn extract(interp: &Artichoke) -> Result<Self, Error> {
        let mut string = <mem::MaybeUninit<sys::mrb_value>>::uninit();
        sys::mrb_get_args(
            interp.0.borrow().mrb,
            Self::ARGSPEC.as_ptr() as *const i8,
            string.as_mut_ptr(),
        );
        let string = string.assume_init();
        if let Ok(string) = interp.try_convert(Value::new(interp, string)) {
            Ok(Self { string })
        } else {
            Err(Error::NoImplicitConversionToString)
        }
    }
}

pub fn method(interp: &Artichoke, args: Args, value: &Value) -> Result<Value, Error> {
    let mrb = interp.0.borrow().mrb;
    let data = unsafe { Regexp::try_from_ruby(interp, value) }.map_err(|_| Error::Fatal)?;
    let string = if let Some(string) = args.string {
        string
    } else {
        unsafe {
            sys::mrb_gv_set(
                mrb,
                interp.0.borrow_mut().sym_intern("$~"),
                sys::mrb_sys_nil_value(),
            );
            return Ok(interp.convert(false));
        }
    };
    let borrow = data.borrow();
    let regex = (*borrow.regex).as_ref().ok_or(Error::Fatal)?;
    let matchdata = match regex {
        Backend::Onig(regex) => {
            if let Some(captures) = regex.captures(string.as_str()) {
                let num_regexp_globals_to_set = {
                    let num_previously_set_globals =
                        interp.0.borrow().num_set_regexp_capture_globals;
                    cmp::max(num_previously_set_globals, captures.len())
                };
                for group in 0..num_regexp_globals_to_set {
                    let sym = if group == 0 {
                        interp.0.borrow_mut().sym_intern("$&")
                    } else {
                        interp.0.borrow_mut().sym_intern(&format!("${}", group))
                    };

                    let value = interp.convert(captures.at(group));
                    unsafe {
                        sys::mrb_gv_set(mrb, sym, value.inner());
                    }
                }
                interp.0.borrow_mut().num_set_regexp_capture_globals = captures.len();

                if let Some(match_pos) = captures.pos(0) {
                    let pre_match = &string[..match_pos.0];
                    let post_match = &string[match_pos.1..];
                    unsafe {
                        let pre_match_sym = interp.0.borrow_mut().sym_intern("$`");
                        sys::mrb_gv_set(mrb, pre_match_sym, interp.convert(pre_match).inner());
                        let post_match_sym = interp.0.borrow_mut().sym_intern("$'");
                        sys::mrb_gv_set(mrb, post_match_sym, interp.convert(post_match).inner());
                    }
                }
                let matchdata = MatchData::new(string.as_str(), borrow.clone(), 0, string.len());
                unsafe { matchdata.try_into_ruby(&interp, None) }.map_err(|_| Error::Fatal)?
            } else {
                unsafe {
                    let pre_match_sym = interp.0.borrow_mut().sym_intern("$`");
                    sys::mrb_gv_set(mrb, pre_match_sym, interp.convert(None::<Value>).inner());
                    let post_match_sym = interp.0.borrow_mut().sym_intern("$'");
                    sys::mrb_gv_set(mrb, post_match_sym, interp.convert(None::<Value>).inner());
                }
                interp.convert(None::<Value>)
            }
        }
        Backend::Rust(_) => unimplemented!("Rust-backed Regexp"),
    };
    unsafe {
        sys::mrb_gv_set(
            mrb,
            interp.0.borrow_mut().sym_intern("$~"),
            matchdata.inner(),
        );
    }
    Ok(interp.convert(!unsafe { sys::mrb_sys_value_is_nil(matchdata.inner()) }))
}
