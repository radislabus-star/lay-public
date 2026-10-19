//! Desktop postcondition of the existing layout intent, never a text executor.
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::task::Poll;
use std::time::{Duration, Instant};

type Layouts = Vec<(String, String, String)>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    owner: String,
    layouts: Layouts,
    index: u32,
}

impl Snapshot {
    fn target(&self, is_ru: bool) -> Result<u32, String> {
        if self.layouts.is_empty()
            || self.layouts.len() > 32
            || self.index as usize >= self.layouts.len()
            || self
                .layouts
                .iter()
                .map(|(a, b, c)| a.len() + b.len() + c.len())
                .sum::<usize>()
                > 8192
        {
            return Err("KDE layout mapping is outside the bounded capability".to_string());
        }
        let id = if is_ru { "ru" } else { "us" };
        let mut indices = self
            .layouts
            .iter()
            .enumerate()
            .filter(|(_, (name, _, _))| name == id);
        let index = indices.next().ok_or("KDE target layout is missing")?.0;
        if indices.next().is_some() {
            return Err("KDE target layout is ambiguous".to_string());
        }
        Ok(index as u32)
    }

    fn verify(&self, observed: &Self, index: u32) -> Result<(), String> {
        if self.owner != observed.owner
            || self.layouts != observed.layouts
            || observed.index != index
        {
            return Err("KDE service, mapping or group changed during layout intent".to_string());
        }
        Ok(())
    }
}

trait Io {
    async fn snapshot(&self) -> Result<Snapshot, String>;
    async fn set_layout(&self, owner: &str, index: u32) -> Result<bool, String>;
}

fn live(is_live: &impl Fn() -> bool) -> Result<(), String> {
    if is_live() {
        Ok(())
    } else {
        Err("KDE layout intent is no longer current".to_string())
    }
}

async fn complete(
    io: &impl Io,
    is_ru: bool,
    is_live: impl Fn() -> bool,
    complete_engine: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    live(&is_live)?;
    let initial = io.snapshot().await?;
    let target = initial.target(is_ru)?;
    initial.verify(&io.snapshot().await?, initial.index)?;
    live(&is_live)?;
    if initial.index != target && !io.set_layout(&initial.owner, target).await? {
        return Err("KDE setLayout returned false".to_string());
    }
    initial.verify(&io.snapshot().await?, target)?;
    live(&is_live)?;
    complete_engine()?;
    live(&is_live)?;
    initial.verify(&io.snapshot().await?, target)?;
    live(&is_live)
}

fn bounded<F: Future>(deadline: Instant, future: F) -> Result<F::Output, String> {
    let mut future = pin!(future);
    let mut timer = pin!(async_io::Timer::at(deadline));
    async_io::block_on(poll_fn(|cx| {
        if Instant::now() >= deadline || timer.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err("KDE layout intent deadline expired".to_string()));
        }
        let result = future.as_mut().poll(cx);
        // The existing, separately bounded IBus CLI may occupy this poll.
        // A result arriving after the deadline is never promoted as success.
        if Instant::now() >= deadline {
            Poll::Ready(Err("KDE layout intent deadline expired".to_string()))
        } else {
            result.map(Ok)
        }
    }))
}

#[cfg(not(test))]
pub(super) fn switch(
    is_ru: bool,
    is_live: impl Fn() -> bool,
    complete_engine: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_millis(180);
    bounded(deadline, async {
        let connection = zbus::Connection::session()
            .await
            .map_err(|e| e.to_string())?;
        complete(&Session(connection), is_ru, is_live, complete_engine).await
    })?
}

#[cfg(not(test))]
struct Session(zbus::Connection);

#[cfg(not(test))]
impl Session {
    async fn owner(&self, name: &str) -> Result<String, String> {
        let message = self
            .0
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "GetNameOwner",
                &(name,),
            )
            .await
            .map_err(|e| e.to_string())?;
        let owner: String = message.body().deserialize().map_err(|e| e.to_string())?;
        zbus::names::UniqueName::try_from(owner.as_str()).map_err(|e| e.to_string())?;
        Ok(owner)
    }

    async fn keyboard_owner(&self) -> Result<String, String> {
        let owner = self.owner("org.kde.keyboard").await?;
        if owner != self.owner("org.kde.KWin").await? {
            return Err("KDE keyboard and KWin service owners differ".to_string());
        }
        Ok(owner)
    }

    async fn call<B: serde::ser::Serialize + zbus::zvariant::DynamicType>(
        &self,
        owner: &str,
        method: &str,
        body: &B,
    ) -> Result<zbus::Message, String> {
        self.0
            .call_method(
                Some(owner),
                "/Layouts",
                Some("org.kde.KeyboardLayouts"),
                method,
                body,
            )
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(not(test))]
impl Io for Session {
    async fn snapshot(&self) -> Result<Snapshot, String> {
        let owner = self.keyboard_owner().await?;
        let layouts: Layouts = self
            .call(&owner, "getLayoutsList", &())
            .await?
            .body()
            .deserialize()
            .map_err(|e| e.to_string())?;
        let index: u32 = self
            .call(&owner, "getLayout", &())
            .await?
            .body()
            .deserialize()
            .map_err(|e| e.to_string())?;
        if owner != self.keyboard_owner().await? {
            return Err("KDE keyboard service restarted during observation".to_string());
        }
        Ok(Snapshot {
            owner,
            layouts,
            index,
        })
    }

    async fn set_layout(&self, owner: &str, index: u32) -> Result<bool, String> {
        self.call(owner, "setLayout", &(index,))
            .await?
            .body()
            .deserialize()
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
#[path = "kde/tests.rs"]
mod tests;
