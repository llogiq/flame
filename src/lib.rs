#[cfg(not(feature = "off"))]
#[macro_use]
extern crate lazy_static;
#[cfg(not(feature = "off"))]
extern crate thread_id;

#[cfg(feature = "json")]
extern crate serde;
#[cfg(feature = "json")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "json")]
extern crate serde_json;

#[cfg(not(feature = "off"))]
mod real;

#[cfg(feature = "off")]
mod empty;

#[cfg(not(feature = "off"))]
pub use real::*;

#[cfg(feature = "off")]
pub use empty::*;

mod html;
