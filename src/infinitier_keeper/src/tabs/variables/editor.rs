//! The variable dialog: a variable's name and value, for a new one (Add) or
//! one of the active tab's (Edit). Save checks them, then writes them; a
//! name or value it can't take keeps the dialog open, saying why.

use infinitier_core::resource::gam::GamVariable;

use super::{Scope, at, variables};
use crate::state::Model;

/// The most characters a variable's name has: the GAM's and the `op187`
/// effect's name fields hold 32.
const NAME_MAX: usize = 32;

/// An open variable dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableEditor {
    scope: Scope,
    /// The variable it edits (see `Variable::address`); `None` for a new
    /// one.
    address: Option<usize>,
    /// The name and value it opened with.
    name: String,
    value: String,
    /// Why the last Save didn't take.
    error: String,
}

impl VariableEditor {
    pub fn title(&self) -> String {
        let verb = match self.address {
            Some(_) => "Edit",
            None => "Add",
        };
        format!("{verb} {} variable", self.scope.noun())
    }

    /// The name and value it opened with, for the dialog's fields.
    pub fn fields(&self) -> (&str, &str) {
        (&self.name, &self.value)
    }

    pub fn error(&self) -> &str {
        &self.error
    }
}

/// Add: open the dialog on a new variable of the active tab's scope.
pub fn add(model: &mut Model) {
    let Some(scope) = Scope::of(model.selected_tab) else {
        return;
    };
    model.variable_editor = Some(VariableEditor {
        scope,
        address: None,
        name: String::new(),
        value: "0".to_owned(),
        error: String::new(),
    });
}

/// Edit: open the dialog on the variable in `row` of the active tab.
pub fn edit(model: &mut Model, row: i32) {
    let Some(scope) = Scope::of(model.selected_tab) else {
        return;
    };
    if let Some(variable) = at(model, scope, row) {
        model.variable_editor = Some(VariableEditor {
            scope,
            address: Some(variable.address),
            name: variable.name,
            value: variable.value.to_string(),
            error: String::new(),
        });
    }
}

/// Save `name` and `value`, as typed: written, and the dialog closed; or,
/// when they won't do, the dialog kept open saying why.
pub fn save(model: &mut Model, name: &str, value: &str) {
    let Some(editor) = model.variable_editor.as_ref() else {
        return;
    };
    let (scope, address) = (editor.scope, editor.address);
    let taken: Vec<String> = variables(model, scope)
        .into_iter()
        .filter(|variable| Some(variable.address) != address)
        .map(|variable| variable.name)
        .collect();
    let (name, value) = match checked(name, value, &taken) {
        Ok(checked) => checked,
        Err(error) => {
            if let Some(editor) = model.variable_editor.as_mut() {
                editor.error = error;
            }
            return;
        }
    };
    match (scope, address) {
        (Scope::Local, Some(index)) => {
            if let Some(cre) = model.selected_cre_mut() {
                cre.set_local_variable(index, &name, value);
            }
        }
        (Scope::Local, None) => {
            if let Some(cre) = model.selected_cre_mut() {
                cre.add_local_variable(&name, value);
            }
        }
        (Scope::Global, Some(index)) => {
            if let Some(variable) = model.gam.variables.get_mut(index) {
                variable.name = name;
                variable.int_value = value;
            }
        }
        (Scope::Global, None) => model.gam.variables.push(GamVariable {
            name,
            // An integer, as every global the games write.
            type_flags: 0,
            ref_value: 0,
            dword_value: 0,
            int_value: value,
            double_value: 0.0,
            script_name: String::new(),
        }),
    }
    model.variable_editor = None;
}

/// Cancel: close, keeping the variable as it was.
pub fn cancel(model: &mut Model) {
    model.variable_editor = None;
}

/// `name` and `value` as a variable takes them: the name trimmed and in
/// capitals (as the games write them), the value a whole number; or why
/// not. `taken` are the other variables' names.
fn checked(name: &str, value: &str, taken: &[String]) -> Result<(String, i32), String> {
    let name = name.trim().to_ascii_uppercase();
    if name.is_empty() {
        return Err("A variable needs a name.".to_owned());
    }
    if !name.chars().all(|c| c.is_ascii_graphic()) {
        return Err("A name is letters, digits and symbols such as _, without spaces.".to_owned());
    }
    if name.len() > NAME_MAX {
        return Err(format!("A name has at most {NAME_MAX} characters."));
    }
    if taken.iter().any(|other| other.eq_ignore_ascii_case(&name)) {
        return Err(format!("There's already a variable named {name}."));
    }
    let value = value.trim().parse::<i32>().map_err(|_| {
        format!(
            "The value is a whole number from {} to {}.",
            i32::MIN,
            i32::MAX
        )
    })?;
    Ok((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taken() -> Vec<String> {
        vec!["CHAPTER".to_owned()]
    }

    #[test]
    fn a_name_is_trimmed_in_capitals_and_a_value_whole() {
        assert_eq!(
            checked(" talked_to_aerie ", "-3", &taken()),
            Ok(("TALKED_TO_AERIE".to_owned(), -3))
        );
    }

    #[test]
    fn a_name_must_be_new_short_and_without_spaces() {
        assert!(checked("", "1", &taken()).is_err());
        assert!(checked("Chapter", "1", &taken()).is_err());
        assert!(checked("TWO WORDS", "1", &taken()).is_err());
        assert!(checked(&"A".repeat(33), "1", &taken()).is_err());
        assert!(checked(&"A".repeat(32), "1", &taken()).is_ok());
    }

    #[test]
    fn a_value_must_be_a_whole_number_that_fits() {
        assert!(checked("X", "1.5", &taken()).is_err());
        assert!(checked("X", "2147483648", &taken()).is_err());
        assert!(checked("X", "", &taken()).is_err());
    }
}
