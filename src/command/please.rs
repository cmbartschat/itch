use crate::{cli::PleaseArgs, ctx::Ctx, error::Attempt, print::show_warning};

pub fn please_command(ctx: &Ctx, args: &PleaseArgs) -> Attempt {
    show_warning(ctx, "You asked me to:");
    args.words.iter().for_each(|word| show_warning(ctx, word));
    /*

    Actions:
    - Save
    - Squash (unsafe)
    - List
    - New
    - Load
    - Prune
    - Merge (unsafe)
    - Delete (unsafe)
    - Status
    - Diff
    - Done

    Initial prompt:

        You are can only respond in a specific format

        List available commands

        The user would like you to:

    For each step:
        Take the response
        Try to parse it
        If it parses, check if the command needs confirmation from the user
            Run the commands
                If the commands end with Done, exit
                If the output is relevant, put the output into the stream
        If it fails to parse, put the message into the stream
        Prompt the AI again
    */
    Ok(())
}
