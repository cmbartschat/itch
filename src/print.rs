use crate::ctx::Ctx;

const RESET: &str = "\x1b[0m";
const FG_ORANGE: &str = "\x1b[38;5;214m";

pub fn show_warning(ctx: &Ctx, message: &str) {
    if ctx.can_prompt() {
        if ctx.color_enabled() {
            eprintln!("{FG_ORANGE}{message}{RESET}");
        } else {
            eprintln!("{message}");
        }
    }
}
