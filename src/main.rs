use std::env;
use std::io::{self, Read};
use std::process::{self, Command};

use keyring_core::{Entry, Error as KeyringError};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("keychain-secret: {error}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let Some(command) = args.get(1).map(String::as_str) else {
        print_usage();
        process::exit(2);
    };

    if matches!(command, "help" | "--help" | "-h") {
        print_usage();
        return Ok(());
    }
    if matches!(command, "--version" | "-V") {
        println!("keychain-secret {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    keyring_core::set_default_store(
        apple_native_keyring_store::keychain::Store::new_with_configuration(
            &std::collections::HashMap::new(),
        )?,
    );

    match command {
        "get" => {
            let (service, account) = parse_service_account(&args)?;
            let entry = Entry::new(service, account)?;
            print!("{}", entry.get_password()?);
            Ok(())
        }
        "set" => {
            let (service, account) = parse_service_account(&args)?;
            let secret = read_secret_from_stdin()?;
            Entry::new(service, account)?.set_password(&secret)?;
            eprintln!("stored {service}/{account}");
            Ok(())
        }
        "delete" => {
            let (service, account) = parse_service_account(&args)?;
            Entry::new(service, account)?.delete_credential()?;
            eprintln!("deleted {service}/{account}");
            Ok(())
        }
        "check" => {
            let (service, account) = parse_service_account(&args)?;
            let entry = Entry::new(service, account)?;
            match entry.get_password() {
                Ok(_) => {
                    println!("present");
                    Ok(())
                }
                Err(KeyringError::NoEntry) => {
                    println!("missing");
                    process::exit(1);
                }
                Err(error) => Err(Box::new(error)),
            }
        }
        "import-op" => {
            let (service, account, op_ref) = parse_import_op_args(&args)?;
            let secret = read_secret_from_op(op_ref)?;
            Entry::new(service, account)?.set_password(&secret)?;
            eprintln!("stored {service}/{account} from 1Password reference");
            Ok(())
        }
        _ => {
            print_usage();
            process::exit(2);
        }
    }
}

fn parse_service_account(args: &[String]) -> Result<(&str, &str)> {
    if args.len() != 4 {
        return Err("expected: <command> <service> <account>".into());
    }
    let service = non_empty("service", &args[2])?;
    let account = non_empty("account", &args[3])?;
    Ok((service, account))
}

fn parse_import_op_args(args: &[String]) -> Result<(&str, &str, &str)> {
    if args.len() != 5 {
        return Err("expected: import-op <service> <account> <op-ref>".into());
    }
    let service = non_empty("service", &args[2])?;
    let account = non_empty("account", &args[3])?;
    let op_ref = non_empty("op-ref", &args[4])?;
    if !op_ref.starts_with("op://") {
        return Err("op-ref must start with op://".into());
    }
    Ok((service, account, op_ref))
}

fn non_empty<'a>(name: &str, value: &'a str) -> Result<&'a str> {
    if value.is_empty() {
        Err(format!("{name} must not be empty").into())
    } else {
        Ok(value)
    }
}

fn read_secret_from_stdin() -> Result<String> {
    let mut secret = String::new();
    io::stdin().read_to_string(&mut secret)?;
    normalize_secret(&mut secret);
    if secret.is_empty() {
        return Err("refusing to store an empty secret".into());
    }
    Ok(secret)
}

fn read_secret_from_op(op_ref: &str) -> Result<String> {
    let output = Command::new("op").arg("read").arg(op_ref).output()?;
    if !output.status.success() {
        // The subprocess may include sensitive values in diagnostics.
        return Err(format!("op read failed with status {}", output.status).into());
    }

    let mut secret = String::from_utf8(output.stdout)?;
    normalize_secret(&mut secret);
    if secret.is_empty() {
        return Err("refusing to store an empty secret from op".into());
    }
    Ok(secret)
}

fn normalize_secret(secret: &mut String) {
    if secret.ends_with('\n') {
        secret.pop();
        if secret.ends_with('\r') {
            secret.pop();
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage:
  keychain-secret --version
  keychain-secret get <service> <account>
  keychain-secret set <service> <account>   # reads secret from stdin
  keychain-secret delete <service> <account>
  keychain-secret check <service> <account>
  keychain-secret import-op <service> <account> <op-ref>"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_removes_only_one_line_ending() {
        for (input, expected) in [
            ("secret\n", "secret"),
            ("secret\r\n", "secret"),
            (" secret \n", " secret "),
            ("secret\n\n", "secret\n"),
            ("secret\r", "secret\r"),
            ("\n", ""),
        ] {
            let mut value = input.to_owned();
            normalize_secret(&mut value);
            assert_eq!(value, expected);
        }
    }

    #[test]
    fn service_and_account_must_be_nonempty_and_exact() {
        let args = ["keychain-secret", "get", "service", "account"].map(str::to_owned);
        assert_eq!(
            parse_service_account(&args).unwrap(),
            ("service", "account")
        );
        assert!(parse_service_account(&args[..3]).is_err());
        let mut empty = args.clone();
        empty[3].clear();
        assert!(parse_service_account(&empty).is_err());
    }

    #[test]
    fn import_requires_an_op_reference() {
        let mut args = [
            "keychain-secret",
            "import-op",
            "service",
            "account",
            "plain-value",
        ]
        .map(str::to_owned);
        assert!(parse_import_op_args(&args).is_err());
        args[4] = "op://vault/item/field".to_owned();
        assert!(parse_import_op_args(&args).is_ok());
    }
}
