use colored::*;
use fs_extra::dir::{get_size, remove};
use glob::glob;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use inquire::{list_option::ListOption, ui::RenderConfig, validator::Validation, MultiSelect};

const SEARCH_PATTERN: &str = "./[!. Library]*/**/node_modules/";
const NONE_FOUND: &str = "Could not find any node_modules!";
const SELECT_QUESTION: &str = "Select which all node_modules you want to delete:";
const SELECT_INVALID: &str = "Select at least one!";
const PROGRESS_TEMPLATE: &str = "{spinner:.yellow} [{bar:50.yellow}] {pos:>7}/{len:7} {msg}";
const PROGRESS_CHARS: &str = "=>-";
const ERROR: &str = "Could not process what to delete!";
const BANNER: &str = r"
        ▐ ▄       ·▄▄▄▄  ▄▄▄ .    ▄ •▄ ▪  ▄▄▌  ▄▄▌  ▄▄▄ .▄▄▄  
        •█▌▐█▪     ██▪ ██ ▀▄.▀·    █▌▄▌▪██ ██•  ██•  ▀▄.▀·▀▄ █·
        ▐█▐▐▌ ▄█▀▄ ▐█· ▐█▌▐▀▀▪▄    ▐▀▀▄·▐█·██▪  ██▪  ▐▀▀▪▄▐▀▀▄ 
        ██▐█▌▐█▌.▐▌██. ██ ▐█▄▄▌    ▐█.█▌▐█▌▐█▌▐▌▐█▌▐▌▐█▄▄▌▐█•█▌
        ▀▀ █▪ ▀█▄▀▪▀▀▀▀▀•  ▀▀▀     ·▀  ▀▀▀▀.▀▀▀ .▀▀▀  ▀▀▀ .▀  ▀
";

fn main() {
    print!("\x1B[2J\x1B[1;1H");

    println!("{}", BANNER.green());

    println!("Searching {}\n", SEARCH_PATTERN.cyan());
    let mut collected: Vec<String> = Vec::new();
    let mut selectable: Vec<String> = Vec::new();

    glob(SEARCH_PATTERN)
        .expect(NONE_FOUND.red().to_string().as_str())
        .for_each(|entry| match entry {
            Ok(path) => {
                let path_str = &path.as_os_str().to_str().unwrap();
                if !collected.iter().any(|f| path_str.contains(f)) {
                    collected.clear();
                    collected.push(path_str.to_string());
                    selectable.push(path_str.to_string());
                }
            }
            _ => (),
        });

    match selectable.len() > 0 {
        true => {
            let ans = MultiSelect::new(SELECT_QUESTION, selectable)
                .with_render_config(RenderConfig {
                    option_index_prefix: inquire::ui::IndexPrefix::ZeroPadded,
                    ..Default::default()
                })
                .with_formatter(&|a| format!("{} folder(s) selected to be deleted.", a.len()))
                .with_validator(|a: &[ListOption<&String>]| {
                    if a.len() < 1 {
                        return Ok(Validation::Invalid(SELECT_INVALID.into()));
                    }
                    Ok(Validation::Valid)
                })
                .prompt();

            match ans {
                Ok(to_del) => {
                    let bar = ProgressBar::new(to_del.len() as u64);
                    let mut size_saved: u64 = 0;

                    bar.set_style(
                        ProgressStyle::with_template(PROGRESS_TEMPLATE)
                            .unwrap()
                            .progress_chars(PROGRESS_CHARS),
                    );

                    for to_delete in &to_del {
                        let size = get_size(to_delete);
                        match remove(to_delete) {
                            Ok(_) => {
                                if size.is_ok() {
                                    size_saved += size.unwrap()
                                }
                                bar.set_message(format!(
                                    "Saved {}",
                                    HumanBytes(size_saved).to_string().green()
                                ));
                            }
                            Err(_) => {
                                bar.set_message(format!("\nCould not delete {}", to_delete.red()))
                            }
                        }
                        bar.inc(1)
                    }

                    bar.finish_with_message(format!("Deleted {} folders!", to_del.len()));
                    println!(
                        "Saved {} by deleting the heaviest known things to man!",
                        HumanBytes(size_saved).to_string().green()
                    )
                }
                Err(_) => println!("{}", ERROR),
            }
        }
        false => println!("{}", NONE_FOUND),
    }
}
