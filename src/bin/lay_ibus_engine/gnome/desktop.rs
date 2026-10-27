use std::future::Future;
use std::task::Poll;
use std::time::Duration;

pub(in crate::window_interaction) fn exact_kitty_window(json: &str) -> bool {
    let Ok(window) = serde_json::from_str::<serde_json::Value>(json) else {
        return false;
    };
    window.get("appId").and_then(|value| value.as_str()) == Some("kitty.desktop")
        && window
            .get("windowId")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty())
        && window
            .get("stableSequence")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty())
}

async fn focused_window_info(deadline: std::time::Instant) -> Option<(String, String)> {
    let mut request = Box::pin(async {
        let connection = zbus::Connection::session().await.ok()?;
        let reply = connection
            .call_method(
                Some("org.gnome.Shell"),
                "/io/github/radislabus_star/LayDaemon",
                Some("io.github.radislabus_star.LayDaemon"),
                "FocusedWindowInfo",
                &(),
            )
            .await
            .ok()?;
        let json = reply.body().deserialize::<String>().ok()?;
        let owner = reply.header().sender()?.as_str().to_owned();
        Some((format!("{}:{owner}", connection.server_guid()), json))
    });
    let mut timeout = Box::pin(async_io::Timer::at(deadline));
    std::future::poll_fn(|cx| {
        if let Poll::Ready(result) = request.as_mut().poll(cx) {
            return Poll::Ready(result);
        }
        if timeout.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}

pub(super) async fn focused_window_is_native_terminal() -> bool {
    focused_window_info(std::time::Instant::now() + Duration::from_millis(150))
        .await
        .is_some_and(|(_, info)| exact_kitty_window(&info))
}

pub(super) async fn focused_window_until(
    deadline: std::time::Instant,
) -> Option<super::FocusedWindow> {
    let (peer, info) = focused_window_info(deadline).await?;
    let parsed: serde_json::Value = serde_json::from_str(&info).ok()?;
    let window = parsed.get("windowId")?.as_str().filter(|v| !v.is_empty())?;
    let sequence = parsed
        .get("stableSequence")?
        .as_str()
        .filter(|v| !v.is_empty())?;
    super::FocusedWindow::from_info(format!("gnome-window:{peer}:{window}:{sequence}"), &info)
}
