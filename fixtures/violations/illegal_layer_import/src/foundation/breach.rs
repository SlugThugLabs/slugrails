//! Foundation module illegally importing from presentation layer.

use crate::cli::presentation_helper;

pub fn foundation_function() {
    presentation_helper();
}
