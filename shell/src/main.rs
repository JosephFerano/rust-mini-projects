use std::env;
use std::io::{stdout, Write, Error, stdin};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn main() -> Result<(), Error> {
    loop {
        print!("$ => ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    let new_dir = args.peekable()
                        .peek()
                        .map_or("/", |x| *x);
                    let path = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(path) {
                        eprintln!("{}", e)
                    }
                    previous_command = None;
                }
                "exit" => { return Ok(()) }
                cmd => {
                    let stdin = previous_command
                        .map_or(Stdio::inherit(), |output: Child| Stdio::from(output.stdout.unwrap()));
                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };
                    let output = Command::new(cmd)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(o) => previous_command = Some(o),
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e)
                        }
                    }
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            final_command.wait()?;
        }
    }
}
