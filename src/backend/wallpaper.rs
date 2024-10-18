use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    backend::{
        request::{Request, RequestImpl},
        MaybeAppID, MaybeWindowIdentifier, Result,
    },
    desktop::{request::ResponseType, wallpaper::SetOn},
    zvariant::{DeserializeDict, OwnedObjectPath, Type},
    AppID, WindowIdentifierType,
};

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct WallpaperOptions {
    #[zvariant(rename = "show-preview")]
    show_preview: Option<bool>,
    #[zvariant(rename = "set-on")]
    set_on: Option<SetOn>,
}

impl WallpaperOptions {
    pub fn show_preview(&self) -> Option<bool> {
        self.show_preview
    }

    pub fn set_on(&self) -> Option<SetOn> {
        self.set_on
    }
}

#[async_trait]
pub trait WallpaperImpl: RequestImpl {
    async fn with_uri(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> Result<()>;
}

pub struct WallpaperInterface {
    imp: Arc<dyn WallpaperImpl>,
    cnx: zbus::Connection,
}

impl WallpaperInterface {
    pub fn new(imp: impl WallpaperImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(imp),
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Wallpaper")]
impl WallpaperInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(name = "SetWallpaperURI")]
    #[zbus(out_args("response"))]
    async fn set_wallpaper_uri(
        &self,
        handle: OwnedObjectPath,
        app_id: MaybeAppID,
        window_identifier: MaybeWindowIdentifier,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> Result<ResponseType> {
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Wallpaper::SetWallpaperURI",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move {
                imp.with_uri(app_id.inner(), window_identifier.inner(), uri, options)
                    .await
            },
        )
        .await
        .map(|r| r.response_type())
    }
}
