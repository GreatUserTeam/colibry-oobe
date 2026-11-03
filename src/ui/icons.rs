//! Brand icons embedded at compile time from the repository's assets/
//! directory: the welcome logo and the ColibryID user pictogram. Every
//! other icon comes from the system GTK icon theme by name.

use gtk4::gdk_pixbuf::prelude::PixbufLoaderExt;
use gtk4::gdk_pixbuf::{Pixbuf, PixbufLoader};

macro_rules! embed {
    ($name:ident, $file:literal) => {
        pub fn $name() -> Pixbuf {
            load(include_bytes!(concat!("../../assets/", $file)))
        }
    };
}

fn load(bytes: &'static [u8]) -> Pixbuf {
    let loader = PixbufLoader::new();
    loader.write(bytes).expect("asset bytes are a valid image");
    loader.close().expect("asset bytes are a complete image");
    loader.pixbuf().expect("asset decodes to a pixbuf")
}

embed!(logo, "logo.png");
embed!(user, "user.png");
