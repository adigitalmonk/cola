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
/// The macro accepts a combination of environment variable to load, identifier to attach it to, and a type to parse it to.
/// Environment variables are always loaded as a String; this will `parse()` the string into the provided type.
///
/// If these values aren't visible at runtime, the process will panic and terminated immediately.
/// We save ourselves from having to write boiler plate to load config and instead can instead just
/// get to work with assurances that the values are there.
///
/// The generated struct also contains some light usage documentation in the rust docs.
///
/// # Examples
///
/// ```
/// // We seed the environment variable for our example
/// std::env::set_var("YOUR_NAME", "Brad");
/// std::env::set_var("YOUR_AGE", "20");
/// // Normally this is done in the system, instead of the application
///
/// cola::make_conf! [
///     "YOUR_NAME" => your_name: String,
///     "YOUR_AGE" => your_age: u32
/// ];

/// impl Configuration {
///     fn hello(&self) -> String {
///         format!("Hello, {}", self.your_name)
///     }

///     fn age(&self) -> String {
///       match self.your_age {
///         age if age >= 18 => {
///           "Voting age".to_string()
///         },
///         age => {
///           "Too young to vote".to_string()
///         }
///       }
///     }
/// }

/// let my_conf = Configuration::default();
/// assert_eq!(my_conf.hello(), "Hello, Brad");
/// assert_eq!(my_conf.age(), "Voting age");
/// 
/// 
/// // This will return an error for if the data was missing.
/// let also_my_conf = Configuration::new().unwrap();
/// assert_eq!(my_conf.your_name, "Brad");
/// assert_eq!(my_conf.your_age, 20u32);
/// ```
///
macro_rules! make_conf {
    ( $( $x:expr => $n:ident: $t:ty ), * ) => {
        use std::str::FromStr;
        type ConfigResult = Result<Configuration, ConfigError>;

        #[derive(Debug)]
        /// Errors that could be raised by the configuration loading process
        enum ConfigError {
            ConfigMissing(String),
            InvalidData(String)
        }

        /// App configuration, wrapped up into a neat package.
        struct Configuration {
            $(
                #[doc="This value represents the data stored in the environment variable "]
                #[doc=$x]
                $n: $t,
            )*
        }

        impl Default for Configuration {
            fn default() -> Configuration {
                match Configuration::new() {
                    Ok(config) => config,
                    Err(ConfigError::ConfigMissing(reason)) => panic!("The value {} is missing", reason),
                    Err(ConfigError::InvalidData(reason)) => panic!("The data stored in {} is invalid", reason)
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
            fn new() -> ConfigResult {
                Ok(Self {
                    $(
                        $n: Self::convert::<$t>(Self::parse_env($x)?)?,
                    )*
                })
            }

            fn convert<T>(source: String) -> Result<T, ConfigError>
            where T: FromStr
            {
                match source.parse::<T>() {
                    Ok(value) => Ok(value),
                    Err(_) => Err(ConfigError::InvalidData(source))
                }
            }

            fn parse_env(key: &str) -> Result<String, ConfigError> {
                match std::env::var(key) {
                    Err(_) => Err(ConfigError::ConfigMissing(String::from(key))),
                    Ok(val) => Ok(val)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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

        make_conf! [
            "TEST_STRING_ENV_KEY" => definitely_new_value: String
        ];

        let conf = Configuration::default();

        assert_eq!(conf.definitely_new_value, "TEST_STRING_VALUE");
    }

    #[test]
    #[should_panic]
    fn it_fails_on_missing_value() {
        make_conf! [
            "DEFINITELY_DOES_NOT_EXIST" => definitely_maybe: String
        ];

        let conf = Configuration::default();

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
        make_conf![
            "DEFINITELY_DOES_NOT_EXIST" => definitely_maybe: String
        ];

        match Configuration::new() {
            Err(ConfigError::ConfigMissing(string)) => assert!(dbg!(string).contains("DEFINITELY_DOES_NOT_EXIST")),
            _ => panic!("should not panic")
        }
    }

    #[test]
    fn invalid_data_returns_apropos_result() {
        #![allow(dead_code)]
        #![allow(clippy::items_after_statements)]

        env::set_var("TEST_TRUE_ENV_KEY", "potato");
        make_conf! ["TEST_TRUE_ENV_KEY" => test_boolean: bool];

        match Configuration::new() {
            Err(ConfigError::InvalidData(string)) => assert!(dbg!(string).contains("potato")),
            Err(err) => panic!("should not panic {err:?}"),
            Ok(_) => panic!("should not be ok")
        }
    }

}
