//! Internal macros shared across `droidkraft-core` feature modules.
//!
//! These reduce the boilerplate of the many "run a fixed shell command" style
//! operations that make up the ADB API surface.

/// Generate `AdbManager` methods that each run a single fixed shell command.
///
/// ```ignore
/// shell_getters! {
///     /// Battery dump.
///     get_battery_info => "dumpsys battery";
///     /// Raw CPU info.
///     get_cpu_info => "cat /proc/cpuinfo";
/// }
/// ```
///
/// Expands to one `pub fn <name>(&mut self) -> AdbResult<String>` per entry,
/// each delegating to [`AdbManager::shell_command`].
#[macro_export]
macro_rules! shell_getters {
    ($(
        $(#[$meta:meta])*
        $name:ident => $cmd:expr;
    )*) => {
        $(
            $(#[$meta])*
            pub fn $name(&mut self) -> $crate::error::AdbResult<String> {
                self.shell_command($cmd)
            }
        )*
    };
}

/// Generate `AdbManager` methods that run a shell command formatted with a
/// single `&str` argument.
///
/// ```ignore
/// shell_arg_ops! {
///     /// Uninstall a package.
///     uninstall_package(package_name) => "pm uninstall {}";
/// }
/// ```
#[macro_export]
macro_rules! shell_arg_ops {
    ($(
        $(#[$meta:meta])*
        $name:ident($arg:ident) => $fmt:expr;
    )*) => {
        $(
            $(#[$meta])*
            pub fn $name(&mut self, $arg: &str) -> $crate::error::AdbResult<String> {
                self.shell_command(&format!($fmt, $arg))
            }
        )*
    };
}
