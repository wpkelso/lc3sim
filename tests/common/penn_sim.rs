use std::{
    env::temp_dir,
    ffi::{OsStr, OsString},
    fs::{self, create_dir_all, read_to_string, File},
    hash::Hash,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use derive_getters::Getters;
use lc3sim_project::{
    defs::LC3Word,
    executors::{populate_from_bin, LC3},
};
use once_map::OnceMap;
use uuid::Uuid;

/// Set of input and result from PennSim.
///
/// Specializes comparisons on the assumption that PennSim assembly output
/// is determinisitic, so only comparing the original asm is sufficient.
#[derive(Debug, Clone, Getters)]
pub struct CompileSet {
    asm: &'static str,
    obj: Box<[u8]>,
    sym: String,
}

impl Hash for CompileSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.asm.hash(state)
    }
}

impl PartialEq for CompileSet {
    fn eq(&self, other: &Self) -> bool {
        self.asm == other.asm
    }
}

impl Eq for CompileSet {}

impl PartialOrd for CompileSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompileSet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.asm.cmp(other.asm)
    }
}

impl CompileSet {
    pub fn new<P: AsRef<Path>>(path: P, data: &'static str) -> Self {
        let path = path.as_ref();
        let file_stem = path.file_stem().unwrap();

        // Create the temporary directory
        let temp_dir = temp_dir().join(Uuid::new_v4().to_string());
        create_dir_all(&temp_dir).unwrap();

        // Create the temp asm file path
        let new_asm_path = temp_dir.join(path.file_name().unwrap());
        let _ = File::create_new(&new_asm_path)
            .unwrap()
            .write_all(data.as_bytes());

        let script_path = temp_dir.join("script");
        let _ = File::create_new(&script_path).unwrap().write_all(
            ("as ".to_string() + new_asm_path.to_str().unwrap() + "\nquit\n").as_bytes(),
        );

        let _ = Command::new("java")
            .args([
                "-jar",
                "penn_sim/PennSim.jar",
                "-t",
                "-s",
                script_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        let to_u8_box = |path| {
            let mut out = Vec::new();
            BufReader::new(File::open(path).unwrap())
                .read_to_end(&mut out)
                .unwrap();
            out.into_boxed_slice()
        };

        let this = Self {
            asm: data,
            obj: to_u8_box(temp_dir.join(PathBuf::from(OsString::from_iter([
                file_stem,
                OsStr::new(".obj"),
            ])))),
            sym: read_to_string(temp_dir.join(PathBuf::from(OsString::from_iter([
                file_stem,
                OsStr::new(".sym"),
            ]))))
            .unwrap(),
        };

        // Clean up the temporary directory
        fs::remove_dir_all(&temp_dir).unwrap();

        this
    }

    pub fn obj_words(&self) -> impl Iterator<Item = LC3Word> + use<'_> {
        let mut bytes = self.obj.bytes();

        let next_pair = move || {
            let first = bytes.next()?.ok()?;
            let second = bytes.next()?.ok()?;
            Some(LC3Word::from_be_bytes([first, second]))
        };

        std::iter::from_fn(next_pair)
    }
}

/// Deliberately leaks values for a static lifetime
static COMPILED: LazyLock<OnceMap<PathBuf, &'static CompileSet>> = LazyLock::new(OnceMap::new);

/// Get this file after processing through PennSim.
///
/// Only compiles a given file through PennSim once, and never drops it.
///
/// See [`static_compiled`] to only provide a filename.
pub fn get_compiled<P: AsRef<Path>>(path: P, data: &'static str) -> &'static CompileSet {
    COMPILED.insert(path.as_ref().to_path_buf(), |_| {
        Box::leak(Box::new(CompileSet::new(path, data)))
    })
}

#[macro_export]
macro_rules! static_compiled {
    ( $x: expr ) => {
        $crate::common::penn_sim::get_compiled($x, include_str!($x))
    };
}

static OS: LazyLock<&'static CompileSet> =
    LazyLock::new(|| static_compiled!("../../penn_sim/lc3os.asm"));

pub fn load_os<P: LC3>(processor: &mut P) {
    populate_from_bin(
        processor,
        &**static_compiled!("../../penn_sim/lc3os.asm").obj(),
    );
}
