use std::time::Duration;

pub const RECONNECT_DELAYS: &[Duration] = &[
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
    Duration::from_secs(8),
    Duration::from_secs(16),
];

pub fn reconnect_with_backoff<T, E>(
    mut connect: impl FnMut() -> Result<T, E>,
    mut sleep: impl FnMut(Duration),
    delays: &[Duration],
) -> Result<T, E> {
    for (index, &delay) in delays.iter().enumerate() {
        let attempt = index + 1;
        tracing::info!(
            event = "discord_reconnect_attempt",
            attempt,
            "Discord reconnect attempt"
        );
        match connect() {
            Ok(connection) => {
                tracing::info!(
                    event = "discord_reconnect_succeeded",
                    attempt,
                    "Discord reconnect succeeded"
                );
                return Ok(connection);
            }
            Err(_) => {
                tracing::warn!(
                    event = "discord_reconnect_attempt_failed",
                    attempt,
                    "Discord reconnect attempt failed"
                );
                sleep(delay);
            }
        }
    }

    let attempt = delays.len() + 1;
    tracing::info!(
        event = "discord_reconnect_attempt",
        attempt,
        "Discord reconnect attempt"
    );
    match connect() {
        Ok(connection) => {
            tracing::info!(
                event = "discord_reconnect_succeeded",
                attempt,
                "Discord reconnect succeeded"
            );
            Ok(connection)
        }
        Err(error) => {
            tracing::error!(
                event = "discord_reconnect_permanently_failed",
                attempt,
                "Discord reconnect permanently failed"
            );
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::reconnect_with_backoff;
    use std::{cell::RefCell, time::Duration};

    #[test]
    fn retries_with_configured_delays_and_returns_replacement_connection() {
        let attempts = RefCell::new(0);
        let delays = RefCell::new(Vec::new());

        let connection = reconnect_with_backoff(
            || {
                let mut attempts = attempts.borrow_mut();
                *attempts += 1;
                if *attempts < 3 {
                    Err("disconnected")
                } else {
                    Ok("replacement connection")
                }
            },
            |delay| delays.borrow_mut().push(delay),
            &[Duration::from_secs(1), Duration::from_secs(2)],
        )
        .unwrap();

        assert_eq!(connection, "replacement connection");
        assert_eq!(*attempts.borrow(), 3);
        assert_eq!(
            *delays.borrow(),
            vec![Duration::from_secs(1), Duration::from_secs(2)]
        );
    }

    #[test]
    fn stops_after_the_bounded_number_of_reconnect_attempts() {
        let attempts = RefCell::new(0);
        let delays = RefCell::new(Vec::new());

        let result = reconnect_with_backoff(
            || {
                *attempts.borrow_mut() += 1;
                Err::<(), _>("disconnected")
            },
            |delay| delays.borrow_mut().push(delay),
            &[Duration::from_secs(1), Duration::from_secs(2)],
        );

        assert_eq!(result, Err("disconnected"));
        assert_eq!(*attempts.borrow(), 3);
        assert_eq!(
            *delays.borrow(),
            vec![Duration::from_secs(1), Duration::from_secs(2)]
        );
    }
}
