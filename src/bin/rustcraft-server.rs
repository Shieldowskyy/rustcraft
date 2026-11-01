use std::process::ExitCode;

const USAGE: &str = "\
RustCraft headless server and relay

Usage:
  rustcraft-server --relay-server [--relay-port PORT]
  rustcraft-server --server [--port PORT] [--seed NUMBER]
";

enum Mode {
    Relay { port: u16 },
    Server { port: u16, seed: u32 },
}

fn parse_args() -> Result<Mode, String> {
    let mut args = std::env::args().skip(1);
    let mut mode = None;
    let mut relay_port = 26657;
    let mut server_port = rustcraft::net::DEFAULT_SERVER_PORT;
    let mut seed = rustcraft::cli::generate_random_seed();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--relay-server" => {
                if mode.replace("relay").is_some() {
                    return Err("Choose either --relay-server or --server.".to_string());
                }
            }
            "--server" => {
                if mode.replace("server").is_some() {
                    return Err("Choose either --relay-server or --server.".to_string());
                }
            }
            "--relay-port" => {
                relay_port = parse_port("--relay-port", args.next())?;
            }
            "--port" => {
                server_port = parse_port("--port", args.next())?;
            }
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--seed requires an integer.".to_string())?;
                seed = value
                    .parse()
                    .map_err(|_| format!("Invalid seed: {value}"))?;
            }
            "--help" | "-h" => return Err(USAGE.to_string()),
            unknown => return Err(format!("Unknown argument: {unknown}\n\n{USAGE}")),
        }
    }

    match mode {
        Some("relay") => Ok(Mode::Relay { port: relay_port }),
        Some("server") => Ok(Mode::Server {
            port: server_port,
            seed,
        }),
        _ => Err(USAGE.to_string()),
    }
}

fn parse_port(name: &str, value: Option<String>) -> Result<u16, String> {
    let value = value.ok_or_else(|| format!("{name} requires a port number."))?;
    let port = value
        .parse::<u16>()
        .map_err(|_| format!("Invalid port for {name}: {value}"))?;
    if port == 0 {
        return Err(format!("Port for {name} must be between 1 and 65535."));
    }
    Ok(port)
}

fn main() -> ExitCode {
    match parse_args() {
        Ok(Mode::Relay { port }) => {
            let address = format!("0.0.0.0:{port}");
            match rustcraft::net::run_relay_server(&address) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("Failed to start RustCraft relay: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(Mode::Server { port, seed }) => {
            rustcraft::net::run_dedicated_server(port, seed);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            if message == USAGE {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_relay_port() {
        assert_eq!(
            parse_port("--relay-port", Some("8080".to_string())).unwrap(),
            8080
        );
    }

    #[test]
    fn rejects_invalid_and_zero_ports() {
        assert!(parse_port("--port", Some("0".to_string())).is_err());
        assert!(parse_port("--port", Some("not-a-port".to_string())).is_err());
        assert!(parse_port("--port", None).is_err());
    }
}
