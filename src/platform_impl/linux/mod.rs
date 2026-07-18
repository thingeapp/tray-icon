// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod icon;
mod menu;
mod tray;

use std::{sync::Arc, thread};

use arc_swap::ArcSwap;

pub(crate) use icon::PlatformIcon;
use tray::Tray;

use crate::{icon::Icon, TrayIconAttributes, TrayIconId};

pub struct TrayIcon {
    tray_handle: ksni::Handle<Tray>,
    update_subscription: u64,
}

fn empty_menu_handle() -> muda::CompatMenuChildrenHandle {
    Arc::new(ArcSwap::from_pointee(Vec::new()))
}

impl TrayIcon {
    pub fn new(id: TrayIconId, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let icon = attrs.icon.map(|icon| icon.inner.into());
        let title = attrs.title.unwrap_or_default();
        let tooltip = attrs.tooltip.unwrap_or_default();

        let menu = attrs
            .menu
            .as_ref()
            .map(|menu| menu.compat_items())
            .unwrap_or_else(empty_menu_handle);

        let tray_service = ksni::TrayService::new(Tray::new(id, icon, title, tooltip, menu));
        let tray_handle = tray_service.handle();
        tray_service.spawn();

        // Each tray gets its own subscription, so update wake-ups are never
        // stolen by another tray. The thread exits when the subscription is
        // dropped (recv returns Err once the sender is unregistered).
        let (update_subscription, update_receiver) = muda::subscribe_menu_update();
        let update_tray_handle = tray_handle.clone();
        thread::spawn(move || {
            while update_receiver.recv().is_ok() {
                update_tray_handle.update(|_| {});
            }
        });

        Ok(Self {
            tray_handle,
            update_subscription,
        })
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) -> crate::Result<()> {
        let icon = icon.map(|icon| icon.inner.into());

        self.tray_handle.update(|tray| {
            tray.set_icon(icon);
        });

        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn crate::menu::ContextMenu>>) {
        let menu = menu
            .as_ref()
            .map(|menu| menu.compat_items())
            .unwrap_or_else(empty_menu_handle);

        self.tray_handle.update(|tray| {
            tray.set_menu(menu);
        });
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        let tooltip = tooltip
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or_default()
            .to_string();

        self.tray_handle.update(|tray| {
            tray.set_tooltip(tooltip);
        });

        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        let title = title
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or_default()
            .to_string();

        self.tray_handle.update(|tray| {
            tray.set_title(title);
        });
    }

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        self.tray_handle.update(|tray| {
            if visible {
                tray.set_status(ksni::Status::Active);
            } else {
                tray.set_status(ksni::Status::Passive);
            }
        });

        Ok(())
    }

    pub fn rect(&self) -> Option<crate::Rect> {
        None
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        // Unregistering drops this tray's update sender, which disconnects the
        // receiver and lets the update thread exit.
        muda::unsubscribe_menu_update(self.update_subscription);
    }
}
