use std::{
    collections::HashMap,
    io::{self, Write},
};

pub fn ask_option(prompt: &str, options: &[&str], default: Option<&str>) -> String {
    eprint!("{prompt} ");

    let last_index = options.len() - 1;
    let second_to_last_index = options.len() - 2;

    let mut shortcut_map: HashMap<String, String> = HashMap::new();
    let mut fullform_map: HashMap<String, String> = HashMap::new();

    if let Some(default) = default {
        shortcut_map.insert("".into(), default.into());
    }

    options.iter().for_each(|f| {
        for i in 1..f.len() {
            let possible_shortcut = &f[0..i];
            if !shortcut_map.contains_key(possible_shortcut) {
                shortcut_map.insert(possible_shortcut.into(), f.to_string());
                fullform_map.insert(f.to_string(), possible_shortcut.into());
                break;
            }
        }
    });

    for (index, option) in options.iter().enumerate() {
        if default == Some(option) {
            eprint!("{} (default)", option);
        } else if let Some(shortcut) = fullform_map.get(*option) {
            eprint!("({}){}", shortcut, &option[shortcut.len()..]);
        }
        if index != last_index {
            if last_index == 1 {
                eprint!(" ");
            } else {
                eprint!(", ");
            }
        }
        if index == second_to_last_index {
            eprint!("or ");
        }
    }

    eprint!(": ");

    loop {
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if options.contains(&input.as_str()) {
            return input;
        }
        if let Some(input) = shortcut_map.get_mut(&input) {
            return input.clone();
        }
        eprintln!("Unrecognized option. Try again:");
    }
}
