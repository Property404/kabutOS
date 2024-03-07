// This is a build script - it runs before any Rust code is compiled
use anyhow::{bail, Result};
use elf::{endian::LittleEndian, ElfStream};
use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

const USERSPACE_DIR: &str = "./src/userspace";
const LINKER_SCRIPT: &str = "./linker.ld";

fn main() -> Result<()> {
    // Set linker script
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg={LINKER_SCRIPT}");
    println!("cargo:rerun-if-changed={LINKER_SCRIPT}");

    // Build userspace
    for krate in ["dratinit"] {
        let file = objcopy(build_crate(krate)?)?;
        let len = file.len();

        let mut contents = format!(
            "// @generated
pub static BIN: [u8; {len}] = [
"
        );
        for byte in file {
            contents += format!("0x{byte:02x},").as_str();
        }
        contents += "];";

        let generated_file_path = format!("{USERSPACE_DIR}/{krate}.rs");
        fs::write(&generated_file_path, contents.as_bytes())?;

        assert!(Command::new("rustfmt")
            .arg(generated_file_path)
            .status()?
            .success());
    }

    Ok(())
}

// Extract bits from an ELF file
fn objcopy(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut elf = ElfStream::<LittleEndian, _>::open_stream(File::open(path)?)?;
    let entry = elf.ehdr.e_entry;
    let mut bytes: Vec<u8> = Vec::new();

    const PROGBITS: u32 = 0x1;
    for sh in elf.section_headers().clone() {
        if sh.sh_type != PROGBITS {
            continue;
        }
        let sh_name: String = elf
            .section_headers_with_strtab()?
            .1
            .unwrap()
            .get(sh.sh_name as usize)?
            .into();
        if sh_name.starts_with(".debug") || sh_name.starts_with(".comment") {
            continue;
        }

        let addr = sh.sh_addr;
        let size = sh.sh_size;
        if addr > 0 && addr < entry {
            bail!("Unexpected start address to section");
        }
        let section = elf.section_data(&sh)?.0;
        if section.len() != usize::try_from(size).unwrap() {
            bail!("Invalid size");
        }
        println!("Section: {sh_name} {size}");
        bytes.extend(section);
    }
    Ok(bytes)
}

// Build a crate and return a path to the binary
fn build_crate(krate: impl AsRef<str>) -> Result<PathBuf> {
    let krate = String::from(krate.as_ref());
    let path = format!("../{krate}");
    in_dir(path.clone(), move || {
        let triple = env::var("TARGET")?;
        let profile = env::var("PROFILE")?;

        // Build
        let status = Command::new("cargo").args(["build"]).status()?;
        if !status.success() {
            bail!("`cargo build` did not exit successfully");
        }

        Ok(PathBuf::from(format!(
            "{path}/target/{triple}/{profile}/{krate}"
        )))
    })
}

// Run some code in a specific directory, then pop bakc
fn in_dir<T>(dir: impl AsRef<Path>, f: impl Fn() -> Result<T>) -> Result<T> {
    let pwd = env::current_dir()?;
    env::set_current_dir(dir)?;
    let result = f();
    env::set_current_dir(pwd)?;
    result
}
