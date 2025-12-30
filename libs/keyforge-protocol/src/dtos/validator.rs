pub trait Validator {
    fn validate(&self) -> Result<(), String>;
}

pub struct LayoutValidator;
impl LayoutValidator {
    pub fn validate_structure(layout: &str) -> Result<(), String> {
        if layout.trim().is_empty() {
            return Err("Layout is empty".to_string());
        }
        if layout.split_whitespace().count() < 10 {
            return Err("Layout has too few keys".to_string());
        }
        Ok(())
    }
}
