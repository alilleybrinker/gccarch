#![deny(missing_docs)]

//! Provides information on GCC's supported architectures.

use bitvec::prelude as bv;
use clap::Parser;
use libc::EXIT_FAILURE;
use nom::{
    bytes::complete::{tag, take},
    character::complete::{alphanumeric1, multispace0},
    combinator::{map, recognize},
    error::ParseError,
    multi::fold_many_m_n,
    sequence::{delimited, separated_pair},
    IResult,
};
use std::fmt::Display;
use std::{fmt, process::exit, result::Result as StdResult};
use thiserror::Error;

fn main() {
    // Report an error nicely if one arises.
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        exit(EXIT_FAILURE);
    }
}

/// Run the program and return an error if something goes wrong.
fn run() -> Result<()> {
    // Parse the CLI arguments.
    let args = &Args::parse();

    // Load the architecture database.
    let arch_db = load_arch_info()?;

    // Decide what to do based on the args.
    match (args.arch.as_ref(), args.feat.as_ref()) {
        // Enforce mutual exclusion of these arguments.
        (Some(_), Some(_)) => Err(Error::ArchAndFeat),

        // Ensure one of them is specified.
        (None, None) => Err(Error::NothingRequested),

        // Print a report about the selected architecture.
        (Some(arch_name), None) => report_arch(arch_name, &arch_db),

        // Print what architectures support a feature.
        (None, Some(feat_name)) => report_feat(feat_name, &arch_db),
    }
}

/// Report on the selected architecture.
fn report_arch(arch_name: &str, arch_db: &[Arch]) -> Result<()> {
    // Get the info for the selected architecture.
    let info = arch_db
        .iter()
        .find(|arch| arch.name == arch_name)
        .map(|arch| &arch.info)
        .ok_or_else(|| Error::unknown_arch(arch_name))?;

    println!("{}", arch_name);
    println!("{}", info.0);

    Ok(())
}

/// Report on the selected feature.
fn report_feat(_feat_name: &str, _arch_db: &[Arch]) -> Result<()> {
    todo!()
}

/// Load the architecture info and parse it.
fn load_arch_info() -> Result<Vec<Arch>> {
    raw_arch_info().map(parse_arch_line).collect()
}

/// Load architecture info from the arch file as an iterator over the lines.
fn raw_arch_info() -> impl Iterator<Item = &'static str> {
    include_str!("arch.txt").lines()
}

/// Parse a single line of the arch file into an arch entry.
fn parse_arch_line(input: &'static str) -> Result<Arch> {
    Ok(map(
        separated_pair(parse_arch_name, tag("| "), parse_arch_info),
        |(name, info)| Arch { name, info },
    )(input)?
    .1)
}

/// Parse the architecture name, ignoring whitespace.
fn parse_arch_name(input: &'static str) -> IResult<&'static str, ArchName> {
    ws(recognize(alphanumeric1))(input)
}

/// Parse information about the architecture.
fn parse_arch_info(input: &'static str) -> IResult<&'static str, ArchInfo> {
    // Get the next character.
    let next_char = take(1usize);

    // Initialize an index tracker and a bitarray initialized to all zeroes.
    let init = || (0, bv::bitarr!(u8, bv::Lsb0; 0; NUM_FIELDS));

    // At each step, update the value in the bitarray and increment the index.
    let step = |(mut idx, mut arch_info): (usize, ArchInfoArray), c| {
        // Hey, turns out for parsing purposes we only care about position!
        arch_info.set(idx, c != " ");
        idx += 1;
        (idx, arch_info)
    };

    // Throw away the index and wrap the bitarray.
    let toss_idx = |(_idx, arch_info)| ArchInfo(arch_info);

    map(
        fold_many_m_n(NUM_FIELDS, NUM_FIELDS, next_char, init, step),
        toss_idx,
    )(input)
}

/// Consume whitespace around the inner parser.
///
/// From: https://docs.rs/nom/latest/nom/recipes/index.html#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

// Type definitions for an architecture entry, which
// consists of the name of the architecture, and a bit
// array representing the facts known about that architecture
// from the arch file.

/// An entry in the arch file.
struct Arch {
    /// The name of the architecture.
    name: ArchName,

    /// The feature information for the architecture.
    info: ArchInfo,
}

impl Arch {
    /// Get if an architecture supports a feature.
    #[allow(unused)]
    fn has_feature(&self, feat: Feat) -> bool {
        // SAFETY: The `Feat` struct is smaller than the limit of the buffer.
        *self.info.0.get(feat as usize).unwrap()
    }
}

/// The name of the architecture as known to GCC.
type ArchName = &'static str;

/// The number of boolean fields represented in the arch file.
///
/// Note this includes the space between the upper and lowercase
/// fields in the arch file, hence the `+ 1` in the definition.
const NUM_FIELDS: usize = 24 + 1;

/// The underlying array storing the architecture info.
type ArchInfoArray = bv::BitArr!(for NUM_FIELDS, in u8);

/// The information known about an architecture by GCC.
struct ArchInfo(ArchInfoArray);

/// The different features supported by the architectures.
#[derive(Debug, Copy, Clone)]
#[allow(unused)]
enum Feat {
    /// A hardware implementation does not exist.
    NoHardwareImpl = 0,

    /// A hardware simulation is not currently being manufactured.
    HardwareImplNotManufactured,

    /// A free simulator does not exist.
    NoFreeSim,

    /// Integer registers are narrower than 32 bits.
    IntRegsLt32B,

    /// Integer registers are at least 64 bits wide.
    IntRegsGte64B,

    /// Memory is not byte addressable, and/or bytes are not eight bits.
    MemNotByteAddrOrNot8B,

    /// Floating point arithmetic is not included in the instruction set.
    NoFloatInInstrSet,

    /// Architecture does not use IEEE format floating point numbers.
    NoIeeeFloat,

    /// Architecture does not have a single condition code register.
    NoSingleCondCodeReg,

    /// Architecture has delay slots.
    HasDelaySlots,

    /// Architecture has a stack that grows upward.
    StackGrowsUp,

    /// Placeholder for the space and should never be used
    #[allow(unused)]
    Ignore,

    /// Port cannot use ILP32 mode integer arithmetic.
    NoIlp32ModeIntArith,

    /// Port can use LP64 mode integer arithmetic.
    HasLp64ModeIntArith,

    /// Port can switch between ILP32 and LP64 at runtime.
    ///
    /// Not necessarily supported by all subtargets.
    SwitchIlp32AndLp64,

    /// Port uses `define_peephole` (as opposed to `define_peephole2`)
    HasDefinePeephole,

    /// Port uses "* ..." notation for output template code.
    StarDotsForOutputTemplates,

    /// Port does not define prologue and/or epilogue RTL expanders.
    NoPrologueEpilogueForRtlExpanders,

    /// Port does not use `define_constants`.
    NoDefineConstants,

    /// Port does not define `TARGET_ASM_FUNCTION_(PRO|EPI)LOGUE`.
    NoDefineTargetAsmFunctionPrologueEpilogue,

    /// Port generates multiple inheritance thunks using
    /// `TARGET_ASM_OUTPUT_MI(_VCALL)_THUNK`.
    MultiInheritanceThunksWithMacro,

    /// Port uses LRA (by default, i.e. unless overriden by a switch).
    PortUsesLraByDefault,

    /// All instructions either produce exactly one assembly instructions,
    /// or trigger a `define_split`.
    InstrsUseOneAsmInstrOrSplit,

    /// `<arch>-elf` is not a supported target.
    ElfNotSupportedTarget,

    /// `<arch>-elf` is the correct target to use with the simulator in
    /// `/cvs/src`.
    ElfCorrectForSim,
}

impl Display for Feat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Feat::NoHardwareImpl => "H",
            Feat::HardwareImplNotManufactured => "M",
            Feat::NoFreeSim => "S",
            Feat::IntRegsLt32B => "L",
            Feat::IntRegsGte64B => "Q",
            Feat::MemNotByteAddrOrNot8B => "N",
            Feat::NoFloatInInstrSet => "F",
            Feat::NoIeeeFloat => "I",
            Feat::NoSingleCondCodeReg => "C",
            Feat::HasDelaySlots => "B",
            Feat::StackGrowsUp => "D",
            Feat::Ignore => " ",
            Feat::NoIlp32ModeIntArith => "l",
            Feat::HasLp64ModeIntArith => "q",
            Feat::SwitchIlp32AndLp64 => "r",
            Feat::HasDefinePeephole => "p",
            Feat::StarDotsForOutputTemplates => "b",
            Feat::NoPrologueEpilogueForRtlExpanders => "f",
            Feat::NoDefineConstants => "m",
            Feat::NoDefineTargetAsmFunctionPrologueEpilogue => "g",
            Feat::MultiInheritanceThunksWithMacro => "i",
            Feat::PortUsesLraByDefault => "a",
            Feat::InstrsUseOneAsmInstrOrSplit => "t",
            Feat::ElfNotSupportedTarget => "e",
            Feat::ElfCorrectForSim => "s",
        };

        write!(f, "{}", s)
    }
}

/// A `Result` type with the error pre-filled.
type Result<T> = StdResult<T, Error>;

/// An error arising from a failed parse with Nom.
type NomErr = nom::Err<nom::error::Error<&'static str>>;

/// Represents the types of error that can go wrong in this program.
#[derive(Debug, Error)]
enum Error {
    /// Indicates a parsing error.
    #[error("a parse error occurred")]
    BadParse(#[from] NomErr),

    /// Indicates an unknown architecture name was requested.
    #[error("'{arch_name}' is not a known architecture")]
    UnknownArch { arch_name: String },

    /// Indicates both --arch and --feat together.
    #[error("can't specify both --arch and --feat together")]
    ArchAndFeat,

    /// Indicates neither --arch or --feat were specified.
    #[error("must specify either --arch or --feat")]
    NothingRequested,
}

impl Error {
    fn unknown_arch(arch_name: &str) -> Error {
        Error::UnknownArch {
            arch_name: arch_name.into(),
        }
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The architecture to ask about.
    #[clap(short, long)]
    arch: Option<String>,

    /// The feature to request architectures that match.
    #[clap(short, long)]
    feat: Option<String>,
}
