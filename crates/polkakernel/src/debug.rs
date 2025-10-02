use crate::libc::*;

macro_rules! as_str {
    ($value: expr, $($symbol: ident,)*) => {
        match $value {
            $($symbol => Some(stringify!($symbol)),)*
            _ => None,
        }
    };
}

macro_rules! wrapper {
    ($type: ident, $underlying_type: ident, $($predefined_values: ident,)*) => {
        pub struct $type(pub $underlying_type);

        impl $type {
            pub const fn as_str(&self) -> Option<&'static str> {
                as_str! {
                    self.0,
                    $($predefined_values,)*
                }
            }
        }

        impl core::fmt::Display for $type {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self.as_str() {
                    Some(s) => f.write_str(s),
                    None => write!(f, concat!(stringify!($type), "({})"), self.0),
                }
            }
        }
    };
}

wrapper! { DirFd, i32, AT_FDCWD, }
wrapper! { Signal, u8,
	SIGHUP, SIGINT, SIGQUIT, SIGILL, SIGTRAP, SIGABRT, SIGBUS,
	SIGFPE, SIGKILL, SIGUSR1, SIGSEGV, SIGUSR2, SIGPIPE, SIGALRM,
	SIGTERM, SIGSTKFLT, SIGCHLD, SIGCONT, SIGSTOP, SIGTSTP, SIGTTIN,
	SIGTTOU, SIGURG, SIGXCPU, SIGXFSZ, SIGVTALRM, SIGPROF, SIGWINCH,
	SIGIO, SIGPWR, SIGSYS,
}
wrapper! { SigMask, u8, SIG_BLOCK, SIG_UNBLOCK, SIG_SETMASK, }
