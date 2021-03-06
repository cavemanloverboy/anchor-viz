use anchor_syn::idl::{Idl, IdlAccount, IdlAccountItem, IdlAccounts};
use anyhow::{Result, anyhow};
use plotters::prelude::*;
use plotters::style::text_anchor::Pos;
use plotters::style::ShapeStyle;
use plotters_backend::text_anchor::{HPos, VPos};
use plotters_backend::{BackendColor, FontStyle};
use std::convert::TryInto;
use std::path::{PathBuf, Path};
 
/// This function and necessary infrastructure was taken and adapted from anchor-lang & anchor-syn.
/// It generates an IDL from the source code of an anchor program and loads it into an Idl struct (anchor-syn).
///
/// By default we set `seeds_feature = false`, `skip_lint = False`.
fn extract_idl(file: &str, seeds_feature: bool, skip_lint: bool) -> Result<Option<Idl>> {
    // defaults to no seeds;
    let file = shellexpand::tilde(file);
    let manifest_from_path = std::env::current_dir()?.join(PathBuf::from(&*file).parent().unwrap());
    let cargo = Manifest::discover_from_path(manifest_from_path)?
        .ok_or_else(|| anyhow!("Cargo.toml not found"))?;
    anchor_syn::idl::file::parse(&*file, cargo.inner.version(), seeds_feature, !skip_lint)
}

/// This struct was taken and adapted from anchor-cli 0.21.0
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest(cargo_toml::Manifest);
/// This struct was taken and adapted from anchor-cli 0.21.0
pub struct WithPath<T> {
    inner: T,
}

impl<T> WithPath<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl Manifest {

    pub fn from_path(p: impl AsRef<Path>) -> Result<Self> {
        cargo_toml::Manifest::from_path(p)
            .map(Manifest)
            .map_err(Into::into)
    }

    // Climbs each parent directory from a given starting directory until we find a Cargo.toml.
    pub fn discover_from_path(start_from: PathBuf) -> Result<Option<WithPath<Manifest>>> {
        let mut cwd_opt = Some(start_from.as_path());

        while let Some(cwd) = cwd_opt {
            for f in std::fs::read_dir(cwd)? {
                let p = f?.path();
                if let Some(filename) = p.file_name() {
                    if filename.to_str() == Some("Cargo.toml") {
                        let m = WithPath::new(Manifest::from_path(&p)?);
                        return Ok(Some(m));
                    }
                }
            }

            // Not found. Go up a directory level.
            cwd_opt = cwd.parent();
        }

        Ok(None)
    }

    pub fn version(&self) -> String {
        match &self.0.package {
            Some(package) => package.version.to_string(),
            _ => "0.0.0".to_string(),
        }
    }
}

/// Given a program-name, generate visualization from the idl extracted by anchor-syn.
/// This function extracts and passses the idl into `visualize(...)` -- the primary backend function.
///
/// If program-name is None, default to current dir name.
///
/// This function assumes you are either in the root dir of an anchor program,
/// e.g. `anchor init my_project` + `cd my_project`,
/// or in the `my_project/programs/my_program` directory.
#[allow(clippy::too_many_arguments, unused_variables)]
pub fn visual(
    program_name: Option<String>,
    width: usize,
    //viz_args: Vec<String>,
) -> Result<()> {
    
    // new anchor-cli feature as of 0.22.0
    const SKIP_LINT: bool = true;

    // Grab IDL
    let workspace_dir = std::env::current_dir()?;


    let idl;
    let mut extracted_idl;
    // If no program_name is provided, try extracting at src/lib.rs without seeds_feature
    if program_name.is_none() {
        extracted_idl = extract_idl("src/lib.rs", false, SKIP_LINT);
        if let Ok(my_idl) = extracted_idl {
            idl = my_idl.unwrap();
        } else {
            // then, try with seeds_feature
            extracted_idl = extract_idl("src/lib.rs", true, SKIP_LINT);
            if let Ok(my_idl) = extracted_idl {
                idl = my_idl.unwrap();
            } else {
                // then try searching in programs/ directory (with no seeds)
                let stem = workspace_dir
                    .file_stem()
                    .unwrap()
                    .to_os_string()
                    .to_str()
                    .expect("invalid workspace")
                    .to_string();
                extracted_idl = extract_idl(&format!("programs/{}/src/lib.rs", stem), false, SKIP_LINT);
                if let Ok(my_idl) = extracted_idl {
                    idl = my_idl.unwrap();
                } else{
                    // then with seeds
                    extracted_idl = extract_idl(&format!("programs/{}/src/lib.rs", stem), true, SKIP_LINT);
                    if let Ok(my_idl) = extracted_idl {
                        idl = my_idl.unwrap();
                    } else {
                        panic!("\n\n\n\nNo program found. Either you are not in an anchor project directory or\nyour ./programs/PROGRAM name must not match your root anchor project directory name.\ncd into your program's directory or try anchorviz -p PROGRAM\n\n\n");
                    }
                }
            }
        }
    } else {
        // if program_name provided then try that (first without seeds)
        extracted_idl =  extract_idl(&format!("programs/{}/src/lib.rs", program_name.as_ref().unwrap()), false, SKIP_LINT);
        if let Ok(my_idl) = extracted_idl{
            idl = my_idl.unwrap();
        } else {
            // with seeds
            extracted_idl =  extract_idl(&format!("programs/{}/src/lib.rs", program_name.as_ref().unwrap()), true, SKIP_LINT);
            if let Ok(my_idl) = extracted_idl{
                idl = my_idl.unwrap();
            } else {
                panic!("\n\n\n\nNo program named {}.\ncd into your program's directory or try anchorviz -p PROGRAM again\n\n\n\n", program_name.as_ref().unwrap())
            }

        }
    }

    let viz_out: String = workspace_dir
        .join(format!("{}.png", idl.name))
        .to_str()
        .unwrap()
        .to_string();

    // Generate visualization
    visualize(idl, &viz_out, width)
}

/// This function takes in an Idl object (from anchor-syn) and and output path,
/// and generates a visualization of the instructions of an anchor program.
fn visualize(idl: Idl, out: &str, width: usize) -> Result<()> {
    // width and height of fig objects
    const BOX_PX_WIDTH: usize = 240;
    const BOX_PX_HEIGHT: usize = 60;
    // width of header for title
    const HEADER_PX_HEIGHT: usize = 100;
    // width of vertical separator
    const SEP_WIDTH: usize = 2;
    // vertical and horizontal size of gap between objects
    const BUFFER_WIDTH: usize = 8;
    // size of title and other text
    const TITLE_SIZE: i32 = 24;
    const TEXT_SIZE: i32 = 20;

    // Find width and height of figure
    // width: total columns = instructions + state methods
    let mut state_methods = match idl.state.clone() {
        Some(idlstate) => idlstate.methods,
        None => vec![],
    };
    let state_name = match idl.state.clone() {
        Some(idlstate) => idlstate.strct.name,
        None => "".to_string(),
    };
    let columns = idl.instructions.len() + state_methods.len();

    // height: Initialize tracker to find largest instruction/state_method
    let mut rows = 0;
    let mut all_instructions = idl.instructions.clone();
    all_instructions.append(&mut state_methods.clone());
    for instruction in all_instructions {
        // count all accounts not in groups
        let accounts = unpack_group(IdlAccounts {
            name: "".to_string(),
            accounts: instruction.accounts.clone(),
        });

        let num_accounts = accounts.len();
        let acct_height = {
            if num_accounts % width == 0 {
                num_accounts / width
            } else {
                num_accounts / width + 1
            }
        };

        // count all signers
        // instruction + group signers
        let signers = accounts
            .iter()
            .fold(0, |acc, x| acc + if x.is_signer { 1 } else { 0 });

        let sign_height = {
            if signers % width == 0 {
                signers / width
            } else {
                signers / width + 1
            }
        }
        .max(1);

        let args = instruction.args.len();
        let arg_height = {
            if args % width == 0 {
                args / width
            } else {
                args / width + 1
            }
        };

        let height = arg_height + acct_height + sign_height;
        rows = rows.max(height);
    }

    // Steps to take
    // 0) Create a canvas to draw on
    // 1) Title and version
    // 2) Populate vertical separator lines
    // 3) Populate anchor instruction names
    // 4) Populate mut accts
    // 5) Populate immut accounts
    // 6) Populate signers
    // 7) populate args

    // 0) Create a canvas to draw on
    let fig_width: u32 = ((BOX_PX_WIDTH + BUFFER_WIDTH) * width * columns
        + BUFFER_WIDTH * columns
        + (columns - 1) * SEP_WIDTH)
        .try_into()
        .unwrap();
    let fig_height: u32 = ((BOX_PX_HEIGHT + BUFFER_WIDTH) * rows
        + HEADER_PX_HEIGHT
        + 3 * BUFFER_WIDTH
        + BOX_PX_HEIGHT
        + 2 * BUFFER_WIDTH)
        .try_into()
        .unwrap();
    let backend = BitMapBackend::new(out, (fig_width, fig_height)).into_drawing_area();
    backend
        .fill(&WHITE)
        .expect("couldn't fill background color");

    // 1) Title and version
    backend
        .draw(&Text::new(
            format!("Anchor Program: {}", idl.name),
            (fig_width as i32 / 2, HEADER_PX_HEIGHT as i32 / 4),
            TextStyle {
                font: FontDesc::new(FontFamily::Monospace, TITLE_SIZE as f64, FontStyle::Bold),
                color: BackendColor {
                    alpha: 1.0,
                    rgb: (0, 0, 0),
                },
                pos: Pos {
                    h_pos: HPos::Center,
                    v_pos: VPos::Center,
                },
            },
        ))
        .expect("couldn't write 'Anchor Program'");
    backend
        .draw(&Text::new(
            format!("Version: {}", idl.version),
            (fig_width as i32 / 2, HEADER_PX_HEIGHT as i32 / 2),
            TextStyle {
                font: FontDesc::new(FontFamily::Monospace, TITLE_SIZE as f64, FontStyle::Normal),
                color: BackendColor {
                    alpha: 1.0,
                    rgb: (0, 0, 0),
                },
                pos: Pos {
                    h_pos: HPos::Center,
                    v_pos: VPos::Center,
                },
            },
        ))
        .expect("couldn't write version");

    // 2) Vertical Separator lines
    for i in 1..columns {
        // Thin Rectangles as vertical separators
        backend
            .draw(&Rectangle::new(
                [
                    (
                        // top left
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            - SEP_WIDTH) as i32,
                        (HEADER_PX_HEIGHT + BUFFER_WIDTH) as i32,
                    ),
                    (
                        // bottom right
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i) as i32,
                        // ((rows+1)*BOX_PX_HEIGHT + HEADER_PX_HEIGHT + (rows + 2)*BUFFER_WIDTH) as i32, // idk why this doesn't work... this is (in principle) fig_height - 3*BUFFER_WIDTH
                        (fig_height as i32 - 3 * BUFFER_WIDTH as i32),
                    ),
                ],
                Into::<ShapeStyle>::into(&BLACK).filled(),
            ))
            .expect("couldn't draw vertical separators");
    }

    // 3) Populate instruction + state method names
    // 4) Populate signers
    // 5) Populate mut accts
    // 6) Populate immut accounts
    // 7) populate args

    // 3) Populate instruction + state method names
    for (i, instruction) in idl.instructions.iter().enumerate() {
        backend
            .draw(&Rectangle::new(
                [
                    (
                        // top left
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2
                            - BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT + BUFFER_WIDTH) as i32,
                    ),
                    (
                        // bottom right
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2
                            + BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT) as i32,
                    ),
                ],
                Into::<ShapeStyle>::into(&RGBColor(255, 200, 200)).filled(),
            ))
            .expect("couldn't draw rect for instruction");
        backend
            .draw(&Text::new(
                "Instruction:".to_string(),
                (
                    ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH) * i
                        + BUFFER_WIDTH
                        + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2)
                        as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT / 3) as i32,
                ),
                TextStyle {
                    font: FontDesc::new(FontFamily::Monospace, TEXT_SIZE as f64, FontStyle::Normal),
                    color: BackendColor {
                        alpha: 1.0,
                        rgb: (0, 0, 0),
                    },
                    pos: Pos {
                        h_pos: HPos::Center,
                        v_pos: VPos::Center,
                    },
                },
            ))
            .expect("couldn't write 'Instruction'");
        backend
            .draw(&Text::new(
                instruction.name.to_string(),
                (
                    ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH) * i
                        + BUFFER_WIDTH
                        + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2)
                        as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + 2 * BOX_PX_HEIGHT / 3) as i32,
                ),
                TextStyle {
                    font: FontDesc::new(FontFamily::Monospace, TEXT_SIZE as f64, FontStyle::Normal),
                    color: BackendColor {
                        alpha: 1.0,
                        rgb: (0, 0, 0),
                    },
                    pos: Pos {
                        h_pos: HPos::Center,
                        v_pos: VPos::Center,
                    },
                },
            ))
            .expect("couldn't write instruction name");
    }
    for (i, state_method) in state_methods.iter().enumerate() {
        let i = i + idl.instructions.len();
        backend
            .draw(&Rectangle::new(
                [
                    (
                        // top left
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2
                            - BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT + BUFFER_WIDTH) as i32,
                    ),
                    (
                        // bottom right
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2
                            + BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT) as i32,
                    ),
                ],
                Into::<ShapeStyle>::into(&RGBColor(255, 200, 200)).filled(),
            ))
            .expect("couldn't draw rect for instruction");
        backend
            .draw(&Text::new(
                "State Method:".to_string(),
                (
                    ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH) * i
                        + BUFFER_WIDTH
                        + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2)
                        as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT / 3) as i32,
                ),
                TextStyle {
                    font: FontDesc::new(FontFamily::Monospace, TEXT_SIZE as f64, FontStyle::Normal),
                    color: BackendColor {
                        alpha: 1.0,
                        rgb: (0, 0, 0),
                    },
                    pos: Pos {
                        h_pos: HPos::Center,
                        v_pos: VPos::Center,
                    },
                },
            ))
            .expect("couldn't write 'State Method'");
        backend
            .draw(&Text::new(
                format!("{}.{}", state_name, state_method.name),
                (
                    ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH) * i
                        + BUFFER_WIDTH
                        + (BOX_PX_WIDTH * width + BUFFER_WIDTH * (width + 1)) / 2)
                        as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + 2 * BOX_PX_HEIGHT / 3) as i32,
                ),
                TextStyle {
                    font: FontDesc::new(FontFamily::Monospace, TEXT_SIZE as f64, FontStyle::Normal),
                    color: BackendColor {
                        alpha: 1.0,
                        rgb: (0, 0, 0),
                    },
                    pos: Pos {
                        h_pos: HPos::Center,
                        v_pos: VPos::Center,
                    },
                },
            ))
            .expect("couldn't write state method name");
    }

    // concat all instructions + methods
    let mut all_instructions = idl.instructions;
    all_instructions.append(&mut state_methods);
    for (i, instruction) in all_instructions.iter().enumerate() {
        let accounts = unpack_group(IdlAccounts {
            name: "".to_string(),
            accounts: instruction.accounts.clone(),
        });

        let inst_signers: Vec<&IdlAccount> = accounts.iter().filter(|&x| x.is_signer).collect();
        let mut signers = 0; // counter
                             // 4) Populate signers
        for &signer in inst_signers.iter() {
            let (l, k) = (signers / width, signers % width);

            backend
                .draw(&Rectangle::new(
                    [
                        (
                            // top left
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * k) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BOX_PX_HEIGHT
                                + BUFFER_WIDTH * (1 + l)
                                + BOX_PX_HEIGHT * (l)) as i32,
                        ),
                        (
                            // bottom right
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * (k + 1)) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BOX_PX_HEIGHT
                                + BUFFER_WIDTH * (1 + l)
                                + BOX_PX_HEIGHT * (l + 1)) as i32,
                        ),
                    ],
                    Into::<ShapeStyle>::into(&RGBColor(0, 255, 163)).filled(),
                ))
                .expect("couldn't draw rect for signer");
            backend
                .draw(&Text::new(
                    "Signer:".to_string(),
                    (
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + BUFFER_WIDTH * (k + 1)
                            + BOX_PX_WIDTH * k
                            + BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT
                            + 2 * BUFFER_WIDTH
                            + BUFFER_WIDTH * (1 + l)
                            + BOX_PX_HEIGHT * (l + 1)
                            + BOX_PX_HEIGHT / 3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal,
                        ),
                        color: BackendColor {
                            alpha: 1.0,
                            rgb: (0, 0, 0),
                        },
                        pos: Pos {
                            h_pos: HPos::Center,
                            v_pos: VPos::Center,
                        },
                    },
                ))
                .expect("couldn't write 'Signer:'");
            backend
                .draw(&Text::new(
                    signer.name.to_string(),
                    (
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + BUFFER_WIDTH * (k + 1)
                            + BOX_PX_WIDTH * k
                            + BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT
                            + 2 * BUFFER_WIDTH
                            + BUFFER_WIDTH * (1 + l)
                            + BOX_PX_HEIGHT * (l + 1)
                            + 2 * BOX_PX_HEIGHT / 3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal,
                        ),
                        color: BackendColor {
                            alpha: 1.0,
                            rgb: (0, 0, 0),
                        },
                        pos: Pos {
                            h_pos: HPos::Center,
                            v_pos: VPos::Center,
                        },
                    },
                ))
                .expect("couldn't write signer");

            signers += 1;
        }

        let signer_offset = {
            if signers % width == 0 {
                signers / width
            } else {
                signers / width + 1
            }
            .max(1)
        };

        let mut accounts_drawn = 0;

        // 5) Populate mut accts
        for account in accounts.clone() {
            if account.is_mut {
                let (l, k) = (accounts_drawn / width, accounts_drawn % width);

                backend
                    .draw(&Rectangle::new(
                        [
                            (
                                // top left
                                ((BOX_PX_WIDTH * width
                                    + SEP_WIDTH
                                    + (1 + width) * BUFFER_WIDTH)
                                    * i
                                    + BUFFER_WIDTH * (k + 1)
                                    + BOX_PX_WIDTH * k) as i32,
                                (HEADER_PX_HEIGHT
                                    + 2 * BUFFER_WIDTH
                                    + BOX_PX_HEIGHT
                                    + BUFFER_WIDTH * (1 + signer_offset + l)
                                    + BOX_PX_HEIGHT * (l + signer_offset))
                                    as i32,
                            ),
                            (
                                // bottom right
                                ((BOX_PX_WIDTH * width
                                    + SEP_WIDTH
                                    + (1 + width) * BUFFER_WIDTH)
                                    * i
                                    + BUFFER_WIDTH * (k + 1)
                                    + BOX_PX_WIDTH * (k + 1))
                                    as i32,
                                (HEADER_PX_HEIGHT
                                    + 2 * BUFFER_WIDTH
                                    + BOX_PX_HEIGHT
                                    + BUFFER_WIDTH * (1 + signer_offset + l)
                                    + BOX_PX_HEIGHT * (l + 1 + signer_offset))
                                    as i32,
                            ),
                        ],
                        Into::<ShapeStyle>::into(&RGBColor(255, 100, 100)).filled(),
                    ))
                    .expect("couldn't draw rect for mutable account");
                backend
                    .draw(&Text::new(
                        "Mutable Account:".to_string(),
                        (
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * k
                                + BOX_PX_WIDTH / 2) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BUFFER_WIDTH * (signer_offset + l)
                                + BOX_PX_HEIGHT * (l + 1 + signer_offset)
                                + BOX_PX_HEIGHT / 3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal,
                            ),
                            color: BackendColor {
                                alpha: 1.0,
                                rgb: (0, 0, 0),
                            },
                            pos: Pos {
                                h_pos: HPos::Center,
                                v_pos: VPos::Center,
                            },
                        },
                    ))
                    .expect("couldn't write 'Mutable Account:'");
                backend
                    .draw(&Text::new(
                        account.name.to_string(),
                        (
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * k
                                + BOX_PX_WIDTH / 2) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BUFFER_WIDTH * (1 + signer_offset + l)
                                + BOX_PX_HEIGHT * (l + 1 + signer_offset)
                                + 2 * BOX_PX_HEIGHT / 3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal,
                            ),
                            color: BackendColor {
                                alpha: 1.0,
                                rgb: (0, 0, 0),
                            },
                            pos: Pos {
                                h_pos: HPos::Center,
                                v_pos: VPos::Center,
                            },
                        },
                    ))
                    .expect("couldn't write mut account name");

                accounts_drawn += 1;
            }
        }

        // 6) Populate immut accts
        for account in &accounts.clone() {
            if !account.is_mut {
                let (l, k) = (accounts_drawn / width, accounts_drawn % width);

                backend
                    .draw(&Rectangle::new(
                        [
                            (
                                // top left
                                ((BOX_PX_WIDTH * width
                                    + SEP_WIDTH
                                    + (1 + width) * BUFFER_WIDTH)
                                    * i
                                    + BUFFER_WIDTH * (k + 1)
                                    + BOX_PX_WIDTH * k) as i32,
                                (HEADER_PX_HEIGHT
                                    + 2 * BUFFER_WIDTH
                                    + BOX_PX_HEIGHT
                                    + BUFFER_WIDTH * (1 + signer_offset + l)
                                    + BOX_PX_HEIGHT * (l + signer_offset))
                                    as i32,
                            ),
                            (
                                // bottom right
                                ((BOX_PX_WIDTH * width
                                    + SEP_WIDTH
                                    + (1 + width) * BUFFER_WIDTH)
                                    * i
                                    + BUFFER_WIDTH * (k + 1)
                                    + BOX_PX_WIDTH * (k + 1))
                                    as i32,
                                (HEADER_PX_HEIGHT
                                    + 2 * BUFFER_WIDTH
                                    + BOX_PX_HEIGHT
                                    + BUFFER_WIDTH * (1 + signer_offset + l)
                                    + BOX_PX_HEIGHT * (l + signer_offset + 1))
                                    as i32,
                            ),
                        ],
                        Into::<ShapeStyle>::into(&RGBColor(3, 225, 255)).filled(),
                    ))
                    .expect("couldn't draw rect for immutable account");
                backend
                    .draw(&Text::new(
                        "Immutable Account:".to_string(),
                        (
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * k
                                + BOX_PX_WIDTH / 2) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BUFFER_WIDTH * (1 + signer_offset + l)
                                + BOX_PX_HEIGHT * (l + 1 + signer_offset)
                                + BOX_PX_HEIGHT / 3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal,
                            ),
                            color: BackendColor {
                                alpha: 1.0,
                                rgb: (0, 0, 0),
                            },
                            pos: Pos {
                                h_pos: HPos::Center,
                                v_pos: VPos::Center,
                            },
                        },
                    ))
                    .expect("couldn't write 'Immutable Account:'");
                backend
                    .draw(&Text::new(
                        account.name.to_string(),
                        (
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * k
                                + BOX_PX_WIDTH / 2) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BUFFER_WIDTH * (1 + signer_offset + l)
                                + BOX_PX_HEIGHT * (l + 1 + signer_offset)
                                + 2 * BOX_PX_HEIGHT / 3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal,
                            ),
                            color: BackendColor {
                                alpha: 1.0,
                                rgb: (0, 0, 0),
                            },
                            pos: Pos {
                                h_pos: HPos::Center,
                                v_pos: VPos::Center,
                            },
                        },
                    ))
                    .expect("couldn't write immut account name");

                accounts_drawn += 1;
            }
        }

        let account_offset = {
            if accounts_drawn % width == 0 {
                accounts_drawn / width
            } else {
                accounts_drawn / width + 1
            }
        };
        let offset = signer_offset + account_offset;

        // 7) Populate args
        for (args_drawn, arg) in instruction.args.iter().enumerate() {
            let (l, k) = (args_drawn / width, args_drawn % width);

            backend
                .draw(&Rectangle::new(
                    [
                        (
                            // top left
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * k) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BOX_PX_HEIGHT
                                + BUFFER_WIDTH * (1 + offset + l)
                                + BOX_PX_HEIGHT * (l + offset)) as i32,
                        ),
                        (
                            // bottom right
                            ((BOX_PX_WIDTH * width
                                + SEP_WIDTH
                                + (1 + width) * BUFFER_WIDTH)
                                * i
                                + BUFFER_WIDTH * (k + 1)
                                + BOX_PX_WIDTH * (k + 1)) as i32,
                            (HEADER_PX_HEIGHT
                                + 2 * BUFFER_WIDTH
                                + BOX_PX_HEIGHT
                                + BUFFER_WIDTH * (1 + offset + l)
                                + BOX_PX_HEIGHT * (l + offset + 1))
                                as i32,
                        ),
                    ],
                    Into::<ShapeStyle>::into(&RGBColor(220, 31, 255)).filled(),
                ))
                .expect("couldn't draw rect for argument");
            backend
                .draw(&Text::new(
                    //"Argument:".to_string(),
                    format!("{:?}:", arg.ty).to_lowercase(),
                    (
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + BUFFER_WIDTH * (k + 1)
                            + BOX_PX_WIDTH * k
                            + BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT
                            + 2 * BUFFER_WIDTH
                            + BUFFER_WIDTH * (1 + offset + l)
                            + BOX_PX_HEIGHT * (l + 1 + offset)
                            + BOX_PX_HEIGHT / 3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal,
                        ),
                        color: BackendColor {
                            alpha: 1.0,
                            rgb: (0, 0, 0),
                        },
                        pos: Pos {
                            h_pos: HPos::Center,
                            v_pos: VPos::Center,
                        },
                    },
                ))
                .expect("couldn't write 'Argument:'");
            backend
                .draw(&Text::new(
                    arg.name.to_string(),
                    (
                        ((BOX_PX_WIDTH * width + SEP_WIDTH + (1 + width) * BUFFER_WIDTH)
                            * i
                            + BUFFER_WIDTH * (k + 1)
                            + BOX_PX_WIDTH * k
                            + BOX_PX_WIDTH / 2) as i32,
                        (HEADER_PX_HEIGHT
                            + 2 * BUFFER_WIDTH
                            + BUFFER_WIDTH * (1 + offset + l)
                            + BOX_PX_HEIGHT * (l + 1 + offset)
                            + 2 * BOX_PX_HEIGHT / 3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal,
                        ),
                        color: BackendColor {
                            alpha: 1.0,
                            rgb: (0, 0, 0),
                        },
                        pos: Pos {
                            h_pos: HPos::Center,
                            v_pos: VPos::Center,
                        },
                    },
                ))
                .expect("couldn't write argument");
        }
    }
    Ok(())
}

/// Takes any nested `account_group` structure, flattens it, and
/// returns all accounts within as a Vec<IdlAccounts>.
fn unpack_group(account_group: IdlAccounts) -> Vec<IdlAccount> {
    let mut v: Vec<IdlAccount> = vec![];

    for account in account_group.accounts.iter() {
        match account {
            IdlAccountItem::IdlAccount(idl_account) => v.push(idl_account.clone()),
            IdlAccountItem::IdlAccounts(idl_accounts) => {
                v.append(&mut unpack_group(idl_accounts.clone()))
            }
        };
    }
    v
}
