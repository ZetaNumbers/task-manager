use std::{ffi::CString, fs, io, ptr, str::FromStr};

use libc::{c_char, pid_t};
use task_manager_types::unix_process::Process;

use helpers::cvt;

fn posix_spawn(program: CString, args: Vec<CString>) -> io::Result<pid_t> {
    use helpers::cvt_nz;
    use std::mem::MaybeUninit;

    let argv: Vec<_> = args.iter().map(|s| s.as_ptr()).collect();

    let mut pid = 0;

    struct PosixSpawnFileActions<'a>(&'a mut MaybeUninit<libc::posix_spawn_file_actions_t>);

    impl Drop for PosixSpawnFileActions<'_> {
        fn drop(&mut self) {
            unsafe {
                libc::posix_spawn_file_actions_destroy(self.0.as_mut_ptr());
            }
        }
    }

    struct PosixSpawnattr<'a>(&'a mut MaybeUninit<libc::posix_spawnattr_t>);

    impl Drop for PosixSpawnattr<'_> {
        fn drop(&mut self) {
            unsafe {
                libc::posix_spawnattr_destroy(self.0.as_mut_ptr());
            }
        }
    }

    unsafe {
        let mut attrs = MaybeUninit::uninit();
        cvt_nz(libc::posix_spawnattr_init(attrs.as_mut_ptr()))?;
        let attrs = PosixSpawnattr(&mut attrs);

        let mut file_actions = MaybeUninit::uninit();
        cvt_nz(libc::posix_spawn_file_actions_init(
            file_actions.as_mut_ptr(),
        ))?;
        let file_actions = PosixSpawnFileActions(&mut file_actions);

        extern "C" {
            static mut environ: *mut *const c_char;
        }
        let envp = environ;
        cvt_nz(libc::posix_spawnp(
            &mut pid,
            program.as_ptr(),
            file_actions.0.as_ptr(),
            attrs.0.as_ptr(),
            argv.as_ptr() as *const _,
            envp as *const _,
        ))?;
        Ok(pid)
    }
}

pub fn process_from_pid(pid: pid_t) -> Option<Process> {
    try {
        let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
        Process::from_str(&stat).expect("process parsing error")
    }
}

pub fn kill(pid: pid_t) -> io::Result<()> {
    cvt(unsafe { libc::kill(pid, libc::SIGKILL) }).map(drop)
}

// pub fn set_priority()

pub fn list_processes() -> Vec<Process> {
    fs::read_dir("/proc")
        .expect("unable to find `/proc`")
        .filter_map(|entry| try {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_dir().then(|| ())?;
            let pid = entry.file_name().to_str()?.parse().ok()?;
            get_process(pid)?
        })
        .collect()
}

pub fn get_process(pid: pid_t) -> Option<Process> {
    process_from_pid(pid)
}

mod helpers {
    use std::io;

    pub trait IsMinusOne {
        fn is_minus_one(&self) -> bool;
    }

    macro_rules! impl_is_minus_one {
        ($($t:ident)*) => ($(impl IsMinusOne for $t {
            fn is_minus_one(&self) -> bool {
                *self == -1
            }
        })*)
    }

    impl_is_minus_one! { i8 i16 i32 i64 isize }

    pub fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
        if t.is_minus_one() {
            Err(io::Error::last_os_error())
        } else {
            Ok(t)
        }
    }

    pub fn cvt_nz(error: libc::c_int) -> io::Result<()> {
        if error == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(error))
        }
    }
}
