use muda::AboutDialog;

use super::tray::Tray;

pub fn muda_to_ksni_menu_item(item: muda::CompatMenuItemHandle) -> ksni::menu::MenuItem<Tray> {
    match &**item.load() {
        muda::CompatMenuItem::Standard(menu_item) => {
            let id = menu_item.id.clone();
            // Check if this is an "about" menu item with metadata
            if menu_item.predefined_item_id.as_deref() == Some("about") {
                if let Some(ref about_metadata) = menu_item.about_metadata {
                    let about_dialog = AboutDialog::from_compat(about_metadata);
                    ksni::menu::StandardItem {
                        label: menu_item.label.clone(),
                        enabled: menu_item.enabled,
                        icon_data: menu_item.icon.clone().unwrap_or_default(),
                        activate: Box::new(move |_| {
                            about_dialog.show();
                        }),
                        ..Default::default()
                    }
                    .into()
                } else {
                    // About without metadata - just send event
                    ksni::menu::StandardItem {
                        label: menu_item.label.clone(),
                        enabled: menu_item.enabled,
                        icon_data: menu_item.icon.clone().unwrap_or_default(),
                        activate: Box::new(move |_| send_menu_event(&id)),
                        ..Default::default()
                    }
                    .into()
                }
            } else {
                // Regular menu item
                ksni::menu::StandardItem {
                    label: menu_item.label.clone(),
                    enabled: menu_item.enabled,
                    icon_data: menu_item.icon.clone().unwrap_or_default(),
                    activate: Box::new(move |_| send_menu_event(&id)),
                    ..Default::default()
                }
                .into()
            }
        }
        muda::CompatMenuItem::Checkmark(check_menu_item) => {
            let id = check_menu_item.id.clone();
            ksni::menu::CheckmarkItem {
                label: check_menu_item.label.clone(),
                enabled: check_menu_item.enabled,
                checked: check_menu_item.checked,
                activate: Box::new(move |_| send_menu_event(&id)),
                ..Default::default()
            }
            .into()
        }
        muda::CompatMenuItem::SubMenu(submenu) => ksni::menu::SubMenu {
            label: submenu.label.clone(),
            enabled: submenu.enabled,
            submenu: submenu
                .submenu
                .load()
                .iter()
                .cloned()
                .map(muda_to_ksni_menu_item)
                .collect(),
            ..Default::default()
        }
        .into(),
        muda::CompatMenuItem::Separator => ksni::menu::MenuItem::Separator,
    }
}

fn send_menu_event(id: &str) {
    muda::MenuEvent::send(muda::MenuEvent {
        id: muda::MenuId(id.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arc_swap::ArcSwap;

    use super::*;

    fn standard(label: &str) -> muda::CompatMenuItemHandle {
        Arc::new(ArcSwap::from_pointee(muda::CompatMenuItem::Standard(
            muda::CompatStandardItem {
                id: label.to_string(),
                label: label.to_string(),
                enabled: true,
                icon: None,
                predefined_item_id: None,
                about_metadata: None,
            },
        )))
    }

    #[test]
    fn maps_nested_submenu_children() {
        let nested: muda::CompatMenuChildrenHandle = Arc::new(ArcSwap::from_pointee(vec![
            standard("one"),
            standard("two"),
            standard("three"),
        ]));
        let submenu = Arc::new(ArcSwap::from_pointee(muda::CompatMenuItem::SubMenu(
            muda::CompatSubMenuItem {
                label: "parent".to_string(),
                enabled: true,
                submenu: nested,
            },
        )));

        match muda_to_ksni_menu_item(submenu) {
            ksni::menu::MenuItem::SubMenu(sub) => {
                assert_eq!(sub.label, "parent");
                assert_eq!(sub.submenu.len(), 3);
            }
            _ => panic!("expected a ksni SubMenu"),
        }
    }
}
