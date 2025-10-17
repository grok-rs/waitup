#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::pub_use,
    clippy::question_mark_used,
    clippy::single_call_fn,
    clippy::mod_module_files,
    clippy::implicit_return,
    clippy::separated_literal_suffix,
    clippy::unseparated_literal_suffix,
    clippy::pub_with_shorthand,
    clippy::pub_without_shorthand,
    clippy::allow_attributes,
    clippy::arbitrary_source_item_ordering,
    clippy::min_ident_chars,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::indexing_slicing,
    clippy::redundant_test_prefix,
    clippy::assertions_on_result_states,
    clippy::absolute_paths,
    clippy::default_numeric_fallback,
    clippy::integer_division,
    clippy::integer_division_remainder_used,
    clippy::float_arithmetic,
    clippy::as_conversions,
    clippy::shadow_unrelated,
    clippy::std_instead_of_core,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::missing_trait_methods,
    clippy::pattern_type_mismatch,
    clippy::ref_patterns,
    clippy::self_named_constructors,
    clippy::wildcard_enum_match_arm,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::unreachable,
    clippy::else_if_without_else,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::single_match_else,
    clippy::match_bool,
    clippy::std_instead_of_alloc,
    clippy::unused_trait_names,
    clippy::str_to_string,
    clippy::string_to_string,
    clippy::iter_over_hash_type,
    clippy::infinite_loop,
    clippy::little_endian_bytes,
    clippy::big_endian_bytes,
    clippy::host_endian_bytes,
    clippy::partial_pub_fields,
    clippy::pub_without_shorthand,
    clippy::pub_with_shorthand,
    clippy::error_impl_error,
    clippy::lossy_float_literal,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::redundant_type_annotations,
    clippy::field_reassign_with_default,
    clippy::clone_on_ref_ptr,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks,
    clippy::mixed_read_write_in_expression,
    clippy::tuple_array_conversions,
    clippy::format_push_string,
    clippy::tests_outside_test_module,
    clippy::as_underscore,
    clippy::deref_by_slicing,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::fn_to_numeric_cast_any,
    clippy::format_collect,
    clippy::four_forward_slashes,
    clippy::get_unwrap,
    clippy::impl_trait_in_params,
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::mixed_attributes_style,
    clippy::mutex_atomic,
    clippy::needless_raw_strings,
    clippy::needless_raw_string_hashes,
    clippy::non_ascii_literal,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::print_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::semicolon_outside_block,
    clippy::shadow_same,
    clippy::shadow_reuse,
    clippy::single_char_lifetime_names,
    clippy::string_lit_chars_any,
    clippy::string_lit_as_bytes,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::separated_literal_suffix,
    clippy::use_debug,
    clippy::verbose_file_reads,
    clippy::wildcard_dependencies,
    clippy::module_name_repetitions,
    clippy::missing_inline_in_public_items,
    clippy::missing_trait_methods,
    clippy::missing_docs_in_private_items,
    clippy::single_call_fn,
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::items_after_statements,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::fn_params_excessive_bools,
    clippy::struct_excessive_bools,
    clippy::if_not_else,
    clippy::inline_always,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::too_many_arguments,
    clippy::unreadable_literal,
    clippy::unused_self,
    clippy::used_underscore_binding,
    clippy::wildcard_imports,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::return_self_not_must_use,
    clippy::semicolon_if_nothing_returned,
    clippy::should_implement_trait,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_wraps,
    clippy::naive_bytecount,
    clippy::needless_collect,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::ptr_as_ptr,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::redundant_else,
    clippy::stable_sort_primitive,
    clippy::string_add_assign,
    clippy::unicode_not_nfc,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unused_async,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::verbose_bit_mask,
    clippy::inconsistent_struct_constructor,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::implicit_clone,
    clippy::cloned_instead_of_copied,
    clippy::copy_iterator,
    clippy::default_trait_access,
    clippy::empty_enum,
    clippy::enum_variant_names,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::ignored_unit_patterns,
    clippy::implicit_clone,
    clippy::implicit_hasher,
    clippy::imprecise_flops,
    clippy::inconsistent_struct_constructor,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::inline_fn_without_body,
    clippy::into_iter_on_ref,
    clippy::invalid_upcast_comparisons,
    clippy::iter_not_returning_iterator,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    unexpected_cfgs,
    clippy::missing_enforced_import_renames,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::negative_feature_names,
    clippy::no_effect_underscore_binding,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ptr_cast_constness,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_feature_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::should_implement_trait,
    clippy::single_match_else,
    clippy::stable_sort_primitive,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::struct_excessive_bools,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::trait_duplication_in_bounds,
    clippy::transmute_ptr_to_ptr,
    clippy::trivially_copy_pass_by_ref,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::unicode_not_nfc,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::unused_async,
    clippy::unused_self,
    clippy::use_debug,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::useless_transmute,
    clippy::verbose_bit_mask,
    clippy::wildcard_dependencies,
    clippy::wildcard_enum_match_arm,
    clippy::wildcard_imports,
    clippy::zero_sized_map_values,
    clippy::else_if_without_else,
    clippy::non_ascii_literal,
    clippy::question_mark_used,
    unfulfilled_lint_expectations,
    reason = "restriction lints contain contradictory rules and overly pedantic restrictions"
)]

//! A robust library for waiting until TCP ports, HTTP endpoints, and services become available.
//!
//! This library provides functionality for testing network connectivity and service availability,
//! with support for DNS resolution, exponential backoff, and multiple connection strategies.
//! Perfect for Docker, Kubernetes, CI/CD pipelines, and microservices orchestration.
//!
//! # Features
//!
//! - **Type Safety**: `NewType` wrappers for ports and hostnames with validation
//! - **Multiple Protocols**: TCP socket connections and HTTP/HTTPS requests
//! - **Flexible Configuration**: Timeouts, retry limits, exponential backoff
//! - **Concurrency Strategies**: Wait for all targets or any target
//! - **Graceful Cancellation**: Cancellation token support for clean shutdown
//! - **Rich Error Context**: Detailed error information with contextual messages
//! - **High Performance**: Optimized for minimal allocations and fast execution
//! - **Comprehensive Testing**: Property-based and parameterized test coverage
//!
//! # Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! waitup = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```
//!
//! # Examples
//!
//! ## Basic TCP Connection Check
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let target = Target::tcp("localhost", 8080)?;
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(30))
//!         .interval(Duration::from_secs(1))
//!         .build();
//!
//!     wait_for_connection(&[target], &config).await?;
//!     println!("Service is ready!");
//!     Ok(())
//! }
//! ```
//!
//! ## HTTP Health Check with Custom Headers
//!
//! ```rust,no_run
//! use waitup::Target;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let target = Target::http_builder(Url::parse("https://api.example.com/health")?)
//!         .status(200)
//!         .auth_bearer("your-api-token")
//!         .content_type("application/json")
//!         .build()?;
//!
//!     let config = waitup::WaitConfig::builder()
//!         .timeout(std::time::Duration::from_secs(60))
//!         .build();
//!
//!     waitup::wait_for_connection(&[target], &config).await?;
//!     println!("API is healthy!");
//!     Ok(())
//! }
//! ```
//!
//! ## Multiple Services with Different Strategies
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let targets = vec![
//!         Target::tcp("database", 5432)?,
//!         Target::tcp("cache", 6379)?,
//!         Target::http_url("https://api.example.com/health", 200)?,
//!     ];
//!
//!     // Wait for ALL services to be ready
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(120))
//!         .wait_for_any(false)
//!         .max_retries(Some(20))
//!         .build();
//!
//!     wait_for_connection(&targets, &config).await?;
//!     println!("All services are ready!");
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Configuration with Cancellation
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//! use tokio::time::sleep;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let target = Target::tcp("slow-service", 8080)?;
//!
//!     let (builder, cancel_token) = WaitConfig::builder()
//!         .timeout(Duration::from_secs(60))
//!         .interval(Duration::from_millis(500))
//!         .with_cancellation();
//!
//!     let config = builder.build();
//!
//!     // Cancel after 10 seconds
//!     let cancel_handle = {
//!         let token = cancel_token.clone();
//!         tokio::spawn(async move {
//!             sleep(Duration::from_secs(10)).await;
//!             token.cancel();
//!         })
//!     };
//!
//!     match wait_for_connection(&[target], &config).await {
//!         Ok(_) => println!("Service is ready!"),
//!         Err(waitup::WaitForError::Cancelled) => println!("Operation was cancelled"),
//!         Err(e) => println!("Error: {}", e),
//!     }
//!
//!     cancel_handle.abort(); // Clean up the cancel task
//!     Ok(())
//! }
//! ```
//!
//! ## Docker Compose Integration
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! /// Wait for services defined in docker-compose.yml
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let services = vec![
//!         Target::tcp("postgres", 5432)?,     // Database
//!         Target::tcp("redis", 6379)?,        // Cache
//!         Target::tcp("elasticsearch", 9200)?, // Search
//!         Target::http_url("http://web:8000/health", 200)?, // Web app
//!     ];
//!
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(300))  // 5 minutes for Docker startup
//!         .interval(Duration::from_secs(2))   // Check every 2 seconds
//!         .max_interval(Duration::from_secs(10)) // Max 10 seconds between retries
//!         .connection_timeout(Duration::from_secs(5)) // 5 second connection timeout
//!         .wait_for_any(false)               // Wait for ALL services
//!         .build();
//!
//!     println!("Waiting for services to be ready...");
//!     wait_for_connection(&services, &config).await?;
//!     println!("All services are ready! Starting application...");
//!     Ok(())
//! }
//! ```

// Module declarations
pub mod async_traits;
pub mod config;
pub mod connection;
pub mod error;
pub mod iterators;
#[macro_use]
pub mod macros;
pub mod presets;
pub mod security;
pub mod target;
pub mod types;
pub mod zero_cost;

// Re-export commonly used types for convenient public API
pub use async_traits::{
    AsyncConnectionStrategy, AsyncRetryStrategy, AsyncTargetChecker, ConcurrentProgressStrategy,
    DefaultTargetChecker, ExponentialBackoffStrategy, LinearBackoffStrategy, WaitForAllStrategy,
    WaitForAnyStrategy,
};
pub use config::WaitConfigBuilder;
pub use connection::{wait_for_connection, wait_for_single_target};
pub use error::{Result, ResultExt, WaitForError};
pub use iterators::{ResultSummary, TargetIterExt, TargetResultIterExt};
pub use security::{RateLimiter, SecurityValidator};
pub use target::{HttpTargetBuilder, TcpTargetBuilder};
pub use types::{
    ConnectionError, Hostname, HttpError, Port, PortCategory, RetryCount, StatusCode, Target,
    TargetResult, WaitConfig, WaitResult,
};
pub use zero_cost::{
    ConstRetryStrategy, DynamicPort, LazyFormat, RegisteredPort, SmallString, StringBuilder,
    TargetDisplay, ValidatedPort, WellKnownPort,
};

// Re-export error_messages for internal use
pub(crate) use error::error_messages;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::match_wildcard_for_single_variants,
    reason = "test code where panics are acceptable"
)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::time::Duration;
    use test_case::test_case;
    use url::Url;

    #[test]
    fn test_target_parse_tcp() {
        let target = Target::parse("localhost:8080", 200).unwrap();
        match target {
            Target::Tcp { host, port } => {
                assert_eq!(host.as_str(), "localhost");
                assert_eq!(port.get(), 8080);
            }
            _ => panic!("Expected TCP target"),
        }
    }

    #[test]
    fn test_target_parse_http() {
        let target = Target::parse("https://example.com/health", 200).unwrap();
        match target {
            Target::Http {
                url,
                expected_status,
                ..
            } => {
                assert_eq!(url.to_string(), "https://example.com/health");
                assert_eq!(expected_status, 200);
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test]
    fn test_target_display() {
        let tcp_target = Target::tcp("localhost", 8080).unwrap();
        assert_eq!(tcp_target.display(), "localhost:8080");

        let url = Url::parse("https://example.com/health").unwrap();
        let http_target = Target::http(url, 200).unwrap();
        assert_eq!(http_target.display(), "https://example.com/health");
    }

    #[test]
    fn test_wait_config_builder() {
        let config = WaitConfig::builder()
            .timeout(Duration::from_secs(60))
            .interval(Duration::from_secs(2))
            .max_interval(Duration::from_secs(30))
            .wait_for_any(true)
            .max_retries(Some(10))
            .build();

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(30));
        assert!(config.wait_for_any);
        assert_eq!(config.max_retries, Some(10));
    }

    #[test]
    fn test_calculate_next_interval() {
        use connection::calculate_next_interval;

        let current = Duration::from_secs(1);
        let max = Duration::from_secs(30);

        let next = calculate_next_interval(current, max);
        assert_eq!(next, Duration::from_millis(1500));

        let large_current = Duration::from_secs(25);
        let next = calculate_next_interval(large_current, max);
        assert_eq!(next, max);
    }

    // Property-based tests for Port validation
    proptest! {
        #[test]
        fn test_port_new_valid_range(port in 1u16..=65535) {
            let result = Port::new(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_new_zero_invalid(port in 0u16..=0) {
            let result = Port::new(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_port_well_known_valid_range(port in 1u16..=1023) {
            let result = Port::system_port(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_well_known_invalid_range(port in 1024u16..=65535) {
            let result = Port::system_port(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_port_registered_valid_range(port in 1024u16..=49151) {
            let result = Port::user_port(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_registered_invalid_low_range(port in 1u16..=1023) {
            let result = Port::user_port(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_port_registered_invalid_high_range(port in 49152u16..=65535) {
            let result = Port::user_port(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_port_dynamic_valid_range(port in 49152u16..=65535) {
            let result = Port::dynamic_port(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_dynamic_invalid_range(port in 1u16..=49151) {
            let result = Port::dynamic_port(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_hostname_validation_alphanumeric(
            hostname in "[a-zA-Z0-9][a-zA-Z0-9\\-]{0,61}[a-zA-Z0-9]"
        ) {
            let result = Hostname::new(hostname);
            assert!(result.is_ok());
        }

        #[test]
        fn test_hostname_validation_too_long(
            hostname in "[a-zA-Z]{254,300}"
        ) {
            let result = Hostname::new(hostname);
            assert!(result.is_err());
        }

        #[test]
        fn test_target_tcp_creation(
            hostname in "[a-zA-Z0-9][a-zA-Z0-9\\-]{0,30}[a-zA-Z0-9]",
            port in 1u16..=65535
        ) {
            let result = Target::tcp(hostname, port);
            assert!(result.is_ok());
        }

        #[test]
        fn test_calculate_next_interval_property(
            current_ms in 1u64..=60000,
            max_ms in 60_000u64..=300_000
        ) {
            let current = Duration::from_millis(current_ms);
            let max = Duration::from_millis(max_ms);

            let next = connection::calculate_next_interval(current, max);

            // Next interval should be greater than current (due to exponential backoff)
            assert!(next >= current);
            // Next interval should not exceed max
            assert!(next <= max);
        }
    }

    // Parameterized tests using test-case
    #[test_case("localhost", 80; "http port")]
    #[test_case("example.com", 443; "https port")]
    #[test_case("127.0.0.1", 22; "ssh port")]
    #[test_case("db.example.com", 5432; "postgres port")]
    fn test_tcp_target_creation(hostname: &str, port: u16) {
        let target = Target::tcp(hostname, port).unwrap();
        match target {
            Target::Tcp { host, port: p } => {
                assert_eq!(host.as_str(), hostname);
                assert_eq!(p.get(), port);
            }
            _ => panic!("Expected TCP target"),
        }
    }

    #[test_case(80, Port::http(); "http port constant")]
    #[test_case(443, Port::https(); "https port constant")]
    #[test_case(22, Port::ssh(); "ssh port constant")]
    #[test_case(5432, Port::postgres(); "postgres port constant")]
    #[test_case(3306, Port::mysql(); "mysql port constant")]
    #[test_case(6379, Port::redis(); "redis port constant")]
    fn test_port_constants(expected: u16, port: Port) {
        assert_eq!(port.get(), expected);
    }

    #[test_case("http://example.com/", 200; "http url")]
    #[test_case("https://api.example.com/health", 200; "https health endpoint")]
    #[test_case("https://example.com:8080/status", 204; "custom port and status")]
    fn test_http_target_parsing(url_str: &str, status: u16) {
        let target = Target::parse(url_str, status).unwrap();
        match target {
            Target::Http {
                url,
                expected_status,
                ..
            } => {
                assert_eq!(url.to_string(), url_str);
                assert_eq!(expected_status, status);
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test_case(""; "empty string")]
    #[test_case("invalid-target"; "missing port")]
    #[test_case("host:"; "empty port")]
    #[test_case("host:abc"; "non-numeric port")]
    #[test_case("host:0"; "zero port")]
    #[test_case("host:65536"; "port too high")]
    fn test_invalid_target_parsing(target_str: &str) {
        let result = Target::parse(target_str, 200);
        assert!(result.is_err());
    }

    #[test_case(""; "empty hostname")]
    #[test_case("-example.com"; "starts with hyphen")]
    #[test_case("example.com-"; "ends with hyphen")]
    #[test_case("ex..ample.com"; "empty label")]
    #[test_case(&"a".repeat(254); "too long")]
    fn test_invalid_hostname_validation(hostname: &str) {
        let result = Hostname::new(hostname);
        assert!(result.is_err());
    }

    #[test_case("192.168.1.1"; "valid ipv4")]
    #[test_case("10.0.0.1"; "valid private ip")]
    #[test_case("255.255.255.255"; "max ipv4")]
    fn test_valid_ipv4_hostname(ip: &str) {
        let result = Hostname::ipv4(ip);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), ip);
    }

    #[test_case("192.168.1"; "incomplete ipv4")]
    #[test_case("192.168.1.1.1"; "too many parts")]
    #[test_case("192.168.256.1"; "octet too high")]
    #[test_case("192.168.abc.1"; "invalid octet")]
    fn test_invalid_ipv4_hostname(ip: &str) {
        let result = Hostname::ipv4(ip);
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_const_constructors() {
        let localhost = Hostname::localhost();
        assert_eq!(localhost.as_str(), "localhost");

        let loopback = Hostname::loopback();
        assert_eq!(loopback.as_str(), "127.0.0.1");

        let loopback_v6 = Hostname::loopback_v6();
        assert_eq!(loopback_v6.as_str(), "::1");

        let any = Hostname::any();
        assert_eq!(any.as_str(), "0.0.0.0");
    }

    #[test]
    fn test_target_convenience_constructors() {
        let localhost_target = Target::localhost(8080).unwrap();
        assert_eq!(localhost_target.hostname(), "localhost");
        assert_eq!(localhost_target.port(), Some(8080));

        let loopback_target = Target::loopback(3000).unwrap();
        assert_eq!(loopback_target.hostname(), "127.0.0.1");
        assert_eq!(loopback_target.port(), Some(3000));

        let loopback_v6_target = Target::loopback_v6(9090).unwrap();
        assert_eq!(loopback_v6_target.hostname(), "::1");
        assert_eq!(loopback_v6_target.port(), Some(9090));
    }

    #[test]
    fn test_tcp_builder_fluent_interface() {
        // Test that builder methods return Self for fluent chaining
        let target = Target::tcp_builder("example.com")
            .unwrap()
            .registered_port(8080)
            .build()
            .unwrap();

        assert_eq!(target.hostname(), "example.com");
        assert_eq!(target.port(), Some(8080));
    }

    #[test]
    fn test_tcp_builder_error_deferred() {
        // Test that validation errors are deferred until build()
        let result = Target::tcp_builder("example.com")
            .unwrap()
            .well_known_port(8080) // Invalid for well-known range
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn safe_tcp_targets_macro() {
        // Test the new safe tcp_targets! macro
        let result = tcp_targets![
            "localhost" => 8080,
            "example.com" => 443,
        ];

        assert!(result.is_ok());
        let targets = result.unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].hostname(), "localhost");
        assert_eq!(targets[0].port(), Some(8080));
    }

    #[test]
    fn safe_tcp_targets_macro_error() {
        // Test that the macro properly propagates errors
        let result = tcp_targets![
            "localhost" => 8080,
            "example.com" => 0, // Invalid port
        ];

        assert!(result.is_err());
    }

    #[test]
    fn safe_http_targets_macro() {
        // Test the new safe http_targets! macro
        let result = http_targets![
            "https://example.com" => 200,
            "http://localhost:8080" => 204,
        ];

        assert!(result.is_ok());
        let targets = result.unwrap();
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn safe_http_targets_macro_error() {
        // Test that the macro properly propagates errors
        let result = http_targets![
            "https://example.com" => 200,
            "invalid-url" => 200, // Invalid URL
        ];

        result.unwrap_err();
    }

    #[test]
    fn test_port_category() {
        // System ports
        let http = Port::http();
        assert_eq!(http.category(), PortCategory::System);

        let ssh = Port::ssh();
        assert_eq!(ssh.category(), PortCategory::System);

        // User ports
        let app_port = Port::new(8080).unwrap();
        assert_eq!(app_port.category(), PortCategory::User);

        let postgres = Port::postgres();
        assert_eq!(postgres.category(), PortCategory::User);

        // Dynamic ports
        let ephemeral = Port::new(50000).unwrap();
        assert_eq!(ephemeral.category(), PortCategory::Dynamic);

        let dynamic = Port::new(65535).unwrap();
        assert_eq!(dynamic.category(), PortCategory::Dynamic);
    }

    #[test]
    fn test_port_category_display() {
        assert_eq!(PortCategory::System.to_string(), "system");
        assert_eq!(PortCategory::User.to_string(), "user");
        assert_eq!(PortCategory::Dynamic.to_string(), "dynamic");
    }

    #[test]
    fn test_port_category_range() {
        assert_eq!(PortCategory::System.range(), (1, 1023));
        assert_eq!(PortCategory::User.range(), (1024, 49151));
        assert_eq!(PortCategory::Dynamic.range(), (49152, 65535));
    }

    #[test]
    fn test_port_category_pattern_matching() {
        let port = Port::http();
        let message = match port.category() {
            PortCategory::System => "Requires elevated privileges",
            PortCategory::User => "Standard application port",
            PortCategory::Dynamic => "Temporary/ephemeral port",
        };
        assert_eq!(message, "Requires elevated privileges");
    }
}
