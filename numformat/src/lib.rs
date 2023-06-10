#![warn(missing_docs)]

//! This library wrappers num_format to format numbers according to the system locale.
//! If a system locale is not available, en is used.

#[cfg(any(unix, windows))]
use lazy_static::lazy_static;
#[cfg(any(unix, windows))]
use num_format::SystemLocale;
use num_format::{Locale, ToFormattedString};

#[cfg(any(unix, windows))]
lazy_static! {
    static ref SYSTEM_LOCALE: Option<SystemLocale> = SystemLocale::default().ok();
}

/// Trait applied to numeric types to add the num_format functions
pub trait NumFormat: Sized {
    /// Formats the number using the system locale, falling back to English
    fn num_format(&self) -> String;
    /// Formats the number using given locale
    fn num_format_with(&self, locale: &Locale) -> String;
    /// Formats the number with a given number of significant digits using the system locale, falling back to English
    fn num_format_sigdig(&self, sig_dig: usize) -> String;
    /// Formats the number with a given number of significant digits using the system locale, falling back to English
    fn num_format_sigdig_with(&self, _sig_dig: usize, locale: &Locale) -> String;
}

macro_rules! gen_int_impl {
    ($type:ty) => {
        impl NumFormat for $type {
            fn num_format(&self) -> String {
                #[cfg(any(unix, windows))]
                match &*SYSTEM_LOCALE {
                    Some(locale) => self.to_formatted_string(locale),
                    None => self.to_formatted_string(&Locale::en),
                }

                #[cfg(not(any(unix, windows)))]
                self.to_formatted_string(&Locale::en)
            }

            fn num_format_with(&self, locale: &Locale) -> String {
                self.to_formatted_string(locale)
            }

            fn num_format_sigdig(&self, _sig_dig: usize) -> String {
                self.num_format()
            }

            fn num_format_sigdig_with(&self, _sig_dig: usize, locale: &Locale) -> String {
                self.num_format_with(locale)
            }
        }
    };
}

macro_rules! gen_flt_impl {
    ($type:ty) => {
        impl NumFormat for $type {
            fn num_format(&self) -> String {
                format_float(*self as f64, None, None)
            }

            fn num_format_with(&self, locale: &Locale) -> String {
                format_float(*self as f64, None, Some(locale))
            }

            fn num_format_sigdig(&self, sig_dig: usize) -> String {
                format_float(*self as f64, Some(sig_dig), None)
            }

            fn num_format_sigdig_with(&self, sig_dig: usize, locale: &Locale) -> String {
                format_float(*self as f64, Some(sig_dig), Some(locale))
            }
        }
    };
}

fn format_float(flt: f64, sig_dig: Option<usize>, locale: Option<&Locale>) -> String {
    let full_str = match sig_dig {
        Some(dig) => {
            let mut prec = 0;

            if dig > 0 {
                let mut num = flt;
                let min_val = 10f64.powf((dig - 1) as f64);

                while num.ceil() < min_val {
                    num *= 10f64;
                    prec += 1;
                }
            }

            format!("{flt:.prec$}")
        }
        None => format!("{flt}"),
    };

    let parts = full_str.split('.').collect::<Vec<&str>>();
    let int_part = parts[0].parse::<i64>().unwrap();

    #[cfg(any(unix, windows))]
    let sys_locale = &*SYSTEM_LOCALE;

    #[cfg(not(any(unix, windows)))]
    let sys_locale: &Option<Locale> = &None;

    let (sep, int_part_str) = match (locale, sys_locale) {
        (Some(locale), _) => (locale.decimal(), int_part.to_formatted_string(locale)),
        (None, Some(locale)) => (locale.decimal(), int_part.to_formatted_string(locale)),
        (None, None) => (
            Locale::en.decimal(),
            int_part.to_formatted_string(&Locale::en),
        ),
    };

    if parts.len() > 1 {
        format!("{}{}{}", int_part_str, sep, parts[1])
    } else {
        int_part_str
    }
}

gen_int_impl!(usize);
gen_int_impl!(u64);
gen_int_impl!(u32);
gen_int_impl!(u16);
gen_int_impl!(u8);

gen_int_impl!(isize);
gen_int_impl!(i64);
gen_int_impl!(i32);
gen_int_impl!(i16);
gen_int_impl!(i8);

gen_flt_impl!(f64);
gen_flt_impl!(f32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intcheck() {
        assert_eq!((-1000i16).num_format_with(&Locale::en), "-1,000");
        assert_eq!((-100i16).num_format_with(&Locale::en), "-100");
        assert_eq!((-10i16).num_format_with(&Locale::en), "-10");
        assert_eq!((-1i16).num_format_with(&Locale::en), "-1");
        assert_eq!(0u16.num_format_with(&Locale::en), "0");
        assert_eq!(1u16.num_format_with(&Locale::en), "1");
        assert_eq!(10u16.num_format_with(&Locale::en), "10");
        assert_eq!(100u16.num_format_with(&Locale::en), "100");
        assert_eq!(1000u16.num_format_with(&Locale::en), "1,000");
    }

    #[test]
    fn fltcheck() {
        assert_eq!((-1000f64).num_format_with(&Locale::en), "-1,000");
        assert_eq!((-100f64).num_format_with(&Locale::en), "-100");
        assert_eq!((-10f64).num_format_with(&Locale::en), "-10");
        assert_eq!((-1f64).num_format_with(&Locale::en), "-1");
        assert_eq!(0f64.num_format_with(&Locale::en), "0");
        assert_eq!(1f64.num_format_with(&Locale::en), "1");
        assert_eq!(10f64.num_format_with(&Locale::en), "10");
        assert_eq!(100f64.num_format_with(&Locale::en), "100");
        assert_eq!(1000f64.num_format_with(&Locale::en), "1,000");

        assert_eq!((0.1f64).num_format_with(&Locale::en), "0.1");
        assert_eq!((0.01f64).num_format_with(&Locale::en), "0.01");
        assert_eq!((0.001f64).num_format_with(&Locale::en), "0.001");
        assert_eq!((0.0001f64).num_format_with(&Locale::en), "0.0001");
        assert_eq!((0.00001f64).num_format_with(&Locale::en), "0.00001");

        assert_eq!((0.3f64).num_format_with(&Locale::en), "0.3");

        assert_eq!(10f64.num_format_sigdig_with(0, &Locale::en), "10", "sig 0");
        assert_eq!(10f64.num_format_sigdig_with(1, &Locale::en), "10", "sig 1");
        assert_eq!(10f64.num_format_sigdig_with(2, &Locale::en), "10", "sig 2");
        assert_eq!(
            10f64.num_format_sigdig_with(3, &Locale::en),
            "10.0",
            "sig 3"
        );
        assert_eq!(
            10f64.num_format_sigdig_with(4, &Locale::en),
            "10.00",
            "sig 4"
        );

        assert_eq!(1.1111.num_format_sigdig_with(0, &Locale::en), "1", "sig 0");
        assert_eq!(1.1111.num_format_sigdig_with(1, &Locale::en), "1", "sig 1");
        assert_eq!(
            1.1111.num_format_sigdig_with(2, &Locale::en),
            "1.1",
            "sig 2"
        );
        assert_eq!(
            1.1111.num_format_sigdig_with(3, &Locale::en),
            "1.11",
            "sig 3"
        );
        assert_eq!(
            1.1111.num_format_sigdig_with(4, &Locale::en),
            "1.111",
            "sig 4"
        );

        assert_eq!(5.5555.num_format_sigdig_with(0, &Locale::en), "6", "sig 0");
        assert_eq!(5.5555.num_format_sigdig_with(1, &Locale::en), "6", "sig 1");
        assert_eq!(
            5.5555.num_format_sigdig_with(2, &Locale::en),
            "5.6",
            "sig 2"
        );
        assert_eq!(
            5.5555.num_format_sigdig_with(3, &Locale::en),
            "5.56",
            "sig 3"
        );
        assert_eq!(
            5.5555.num_format_sigdig_with(4, &Locale::en),
            "5.556",
            "sig 4"
        );

        assert_eq!(9.9999.num_format_sigdig_with(0, &Locale::en), "10", "sig 0");
        assert_eq!(9.9999.num_format_sigdig_with(1, &Locale::en), "10", "sig 1");
        assert_eq!(9.9999.num_format_sigdig_with(2, &Locale::en), "10", "sig 2");
        assert_eq!(
            9.9999.num_format_sigdig_with(3, &Locale::en),
            "10.0",
            "sig 3"
        );
        assert_eq!(
            9.9999.num_format_sigdig_with(4, &Locale::en),
            "10.00",
            "sig 4"
        );
    }
}
