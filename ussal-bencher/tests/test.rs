#[cfg(not(target_os = "windows"))]
mod test {
    use serial_test::serial;
    use std::time::Duration;
    use tokio_bin_process::event::Level;
    use tokio_bin_process::event_matcher::EventMatcher;
    use tokio_bin_process::BinProcess;

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_invalid_auth_key() {
        let runner = ussal_server().await;

        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        if std::process::Command::new(&cargo)
            .args([
                "run",
                "-p",
                "ussal-bencher",
                "--",
                "--address",
                "ws://localhost:8000/run_job",
                "--auth-token",
                // TOOD: better error reporting for incorrect auth key
                // TODO: separate test with valid auth key
                "2d58efc6-6c95-47c5-968d-55aa923b4cc9",
            ])
            .status()
            .unwrap()
            .success()
        {
            // TODO: assert on output
            panic!("ussal-bencher returned succesful exit code")
        }

        runner.shutdown_and_then_consume_events(&[]).await;
    }

    async fn ussal_server() -> BinProcess {
        // TODO: run as ussal-server user, probably create a wrapper script that tokio-bin-process runs
        let mut runner = BinProcess::start_with_args(
            "ussal-server",
            "server",
            &[
                "--mode",
                "orchestrator-and-runner",
                "--disable-https",
                "--domains",
                "deletethis",
                "--log-format",
                "json",
            ],
        )
        .await;

        tokio::time::timeout(
            Duration::from_secs(30),
            runner.wait_for(
                &EventMatcher::new()
                    .with_level(Level::Info)
                    .with_target("ussal_server")
                    .with_message("Starting HTTP on port: 8000"),
            ),
        )
        .await
        .unwrap();
        runner
    }
}
