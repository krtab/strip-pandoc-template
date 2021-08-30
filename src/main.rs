use itertools::Itertools;
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    template_path: PathBuf,
    variables_to_strip: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
enum Keyword {
    Else,
    If,
    Endif,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt = Opt::from_args();
    let variables_file_content = fs::read_to_string(opt.variables_to_strip)?;
    let variables: HashMap<String, bool> = toml::from_str(&variables_file_content)?;

    let re_if = Regex::new(r"\$if\(([\w-]+)\)\$\n?")?;
    let re_endif = Regex::new(r"\$endif\$\n?")?;
    let re_else = Regex::new(r"\$else\$\n?")?;

    let template_file_content = fs::read_to_string(opt.template_path)?;

    let it = re_if
        .find_iter(&template_file_content)
        .map(|x| (x, Keyword::If))
        .merge_by(
            re_endif
                .find_iter(&template_file_content)
                .map(|x| (x, Keyword::Endif)),
            |m1, m2| m1.0.start() < m2.0.start(),
        )
        .merge_by(
            re_else
                .find_iter(&template_file_content)
                .map(|x| (x, Keyword::Else)),
            |m1, m2| m1.0.start() < m2.0.start(),
        );
    let mut stack: Vec<Option<bool>> = Vec::new();
    let mut to_process = template_file_content.as_str();
    let mut processed = 0;
    for (m, kwd) in it {
        // dbg!(&to_process[..10]);
        let (previous_content, rem) = to_process.split_at(m.start() - processed);
        // dbg!(&previous_content.chars().last());
        // dbg!(&rem[..4]);
        if stack.iter().all(|&x| x.unwrap_or(true)) {
            print!("{}", previous_content);
        }
        processed += previous_content.len();

        let print_instr = match kwd {
            Keyword::If => {
                let outter_context_printing = stack.iter().all(|&x| x.unwrap_or(true));
                let var_name = re_if.captures(m.as_str()).unwrap().get(1).unwrap().as_str();
                let var_value = variables.get(var_name).copied();
                stack.push(var_value);
                outter_context_printing && var_value.is_none()
            }
            Keyword::Else => {
                let was_stripping = stack.pop().unwrap();
                let outter_context_printing = stack.iter().all(|&x| x.unwrap_or(true));
                stack.push(was_stripping.map(|x| !x));
                outter_context_printing && was_stripping.is_none()
            }
            Keyword::Endif => {
                let was_stripping = stack.pop().unwrap();
                let outter_context_printing = stack.iter().all(|&x| x.unwrap_or(true));
                outter_context_printing && was_stripping.is_none()
            }
        };
        let (instr, rem) = rem.split_at(m.end() - processed);
        // dbg!(&instr.chars().last());
        // dbg!(&rem[..4]);
        if print_instr {
            print!("{}", instr);
        }
        processed += instr.len();
        to_process = rem;

        // println!(">>>> {} | {:?}", m.as_str(), &stack)
    }
    print!("{}", to_process);
    Ok(())
}
