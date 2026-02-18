use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match eden_skills_cli::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(eden_skills_cli::exit_code_for_error(&err))
        }
    }
}
