//! Loading configuration can be annoying.
//! This project is meant to save you on some boilerplate.

#![deny(clippy::correctness)]
#![deny(clippy::nursery)]
#![deny(clippy::suspicious)]
#![deny(clippy::pedantic)]
#![deny(clippy::complexity)]
#![warn(clippy::style)]

#[macro_export]
/// A macro that generates the Configuration struct.
///
/// The macro accepts a combination of environment variables to load, identifier to attach it to, and a type to parse it into.
/// Environment variables are always loaded as a String; this will call <core::str::parse> on the string into the provided type.
/// As a result, whatever your target data type is must implement <core::str::FromStr>.
///
/// The struct will expose two methods:
/// - a `new` method that will return a Result with the loaded data; the error is sbased on what is wrong when loading
/// - a `default` implementation that will return the structure directly; it will call `std::panic` if anything is wrong
///
/// We save ourselves from having to write boiler plate to load config and instead can instead just
/// get to work with assurances that the values are there.
///
/// The generated struct also contains some light usage documentation for your rust docs.
///
/// # Visibility
///
/// The generated configuration will be entirely public to allow for simple extension;
/// it's recommended you wrap the configuration into it's own module to limit the visibility.
///
/// # Examples
///
/// ```
/// // We seed the environment variable for our example
/// std::env::set_var("EX_YOUR_NAME", "Brad");
/// std::env::set_var("EX_YOUR_AGE", "20");
/// // This should be done at the system level instead of the application
///
/// mod my_conf {
///     cola::make_conf! [
///         "EX_YOUR_NAME" => your_name: String,
///         "EX_YOUR_AGE" => your_age: u32
///     ];

///     impl Configuration {
///         pub fn hello(&self) -> String {
///             format!("Hello, {}", self.your_name)
///         }

///         pub fn age(&self) -> String {
///             match self.your_age {
///                 age if age >= 18 => "Voting age".to_string(),
///                 age => "Too young to vote".to_string()
///             }
///         }
///     }
/// }
///
/// use my_conf::*;
///
/// let my_conf = Configuration::default();
/// assert_eq!(my_conf.hello(), "Hello, Brad");
/// assert_eq!(my_conf.age(), "Voting age");
///
/// // If you want control over the bad data situation
/// use cola::ConfigError;
/// let also_my_conf = match Configuration::new() {
///     Ok(conf) => conf,
///     Err(ConfigError::ConfigMissing(reason)) => panic!("'{reason}' not found"),
///     Err(ConfigError::InvalidData(reason)) => panic!("'{reason}' not parseable")
/// };
/// assert_eq!(also_my_conf.your_name, "Brad");
/// assert_eq!(also_my_conf.your_age, 20u32);
/// ```
///
macro_rules! make_conf {
    ( $( $x:expr => $n:ident: $t:ty ), * ) => {
        use $crate::ConfigError;

        /// App configuration, wrapped up into a neat package.
        pub struct Configuration {
            $(
                #[doc="This value represents the data stored in the environment variable "]
                #[doc=$x]
                pub $n: $t,
            )*
        }

        impl Default for Configuration {
            fn default() -> Configuration {
                match Configuration::new() {
                    Ok(config) => config,
                    Err(ConfigError::ConfigMissing(reason)) => panic!("The value {reason} is missing"),
                    Err(ConfigError::InvalidData(reason)) => panic!("The data stored in {reason} is non-parseable")
                }
            }
        }

        impl Configuration {
            /// Loads application configuration.
            $(
                /// (
                #[doc = $x]
                /// )
            )*
            ///
            pub fn new() -> Result<Configuration, ConfigError> {
                Ok(Self {
                    $(
                        $n: $crate::convert::<$t>($crate::parse_env($x)?)?,
                    )*
                })
            }
        }
    }
}

#[derive(Debug)]
/// Errors that could be raised while loading the configuration
pub enum ConfigError {
    ConfigMissing(String),
    InvalidData(String),
}

/// Convert a String into a given type.
///
/// # Errors
/// - <ConfigError::InvalidData>
pub fn convert<T>(source: String) -> Result<T, ConfigError>
where
    T: core::str::FromStr,
{
    source
        .parse::<T>()
        .map_or_else(|_| Err(ConfigError::InvalidData(source)), Ok)
}

/// Load the data stored in a given environment variable.
///
/// # Errors
/// - <ConfigError::ConfigMissing>
pub fn parse_env(key: &str) -> Result<String, ConfigError> {
    std::env::var(key).map_or_else(
        |_| {
            let error_msg = String::from(key);
            Err(ConfigError::ConfigMissing(error_msg))
        },
        Ok,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    make_conf! [
        "TEST_STRING_ENV_KEY" => test_string: String,
        "TEST_INT_ENV_KEY" => test_number: u32,
        "TEST_INT_ENV_KEY" => test_float_num: f32,
        "TEST_NEG_ENV_KEY" => test_neg_num: i32,
        "TEST_TRUE_ENV_KEY" => test_boolean: bool,
        "TEST_FALSE_ENV_KEY" => test_false_boolean: bool
    ];

    #[test]
    fn it_makes_the_conf_struct_with_expected_types() {
        env::set_var("TEST_STRING_ENV_KEY", "TEST_STRING_VALUE");
        env::set_var("TEST_INT_ENV_KEY", "1");
        env::set_var("TEST_NEG_ENV_KEY", "-1");
        env::set_var("TEST_TRUE_ENV_KEY", "true");
        env::set_var("TEST_FALSE_ENV_KEY", "false");

        let conf = Configuration::default();

        assert_eq!(conf.test_string, "TEST_STRING_VALUE");
        assert_eq!(conf.test_number, 1);
        assert!(conf.test_float_num.eq(&1.0));
        assert_eq!(conf.test_neg_num, -1);
        assert!(conf.test_boolean);
        assert!(!conf.test_false_boolean);
    }

    #[test]
    fn shadows_other_configurations() {
        #![allow(clippy::items_after_statements)]

        env::set_var("TEST_STRING_ENV_KEY", "TEST_STRING_VALUE");

        make_conf! ["TEST_STRING_ENV_KEY" => definitely_new_value: String];

        let conf = Configuration::default();

        assert_eq!(conf.definitely_new_value, "TEST_STRING_VALUE");
    }

    #[test]
    #[should_panic]
    fn it_fails_on_missing_value() {
        mod sub {
            make_conf! ["DEFINITELY_DOES_NOT_EXIST" => definitely_maybe: String];
        }

        let conf = sub::Configuration::default();

        assert_eq!(conf.definitely_maybe, "won't get here");
    }

    #[test]
    fn it_allows_calling_new_directly() {
        #![allow(clippy::items_after_statements)]
        env::set_var("TEST_TRUE_ENV_KEY", "true");
        make_conf! ["TEST_TRUE_ENV_KEY" => test_boolean: bool];

        let conf = Configuration::new().unwrap();
        assert!(conf.test_boolean);
    }

    #[test]
    fn missing_data_returns_apropos_result() {
        #![allow(dead_code)]
        make_conf! ["NOT_FOUND" => definitely_maybe: String];

        match Configuration::new() {
            Err(ConfigError::ConfigMissing(str)) => assert!(str.contains("NOT_FOUND")),
            _ => panic!("should not panic"),
        }
    }

    #[test]
    fn invalid_data_returns_apropos_result() {
        #![allow(dead_code)]
        #![allow(clippy::items_after_statements)]

        env::set_var("TEST_TRUE_ENV_KEY", "potato");
        make_conf! ["TEST_TRUE_ENV_KEY" => test_boolean: bool];

        match Configuration::new() {
            Err(ConfigError::InvalidData(string)) => assert!(string.contains("potato")),
            Err(err) => panic!("should not panic {err:?}"),
            Ok(_) => panic!("should not be ok"),
        }
    }
}
