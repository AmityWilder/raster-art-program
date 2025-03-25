use std::{cell::RefCell, rc::{Rc, Weak}};
use raylib::prelude::*;

pub type Raster = RenderTexture2D;
pub type RcRaster = Rc<RefCell<Raster>>;
pub type WeakRaster = Weak<RefCell<Raster>>;
