use anyhow::*;
use eww_shared_util::VarName;
use std::{collections::HashMap, path::Path};
use yuck::config::{
    file_provider::YuckFiles, script_var_definition::ScriptVarDefinition, widget_definition::WidgetDefinition, Config,
};

use simplexpr::dynval::DynVal;

use crate::error_handling_ctx;

use super::{script_var, EwwWindowDefinition};

/// Load an [EwwConfig] from a given file, resetting and applying the global YuckFiles object in [error_handling_ctx].
pub fn read_from_file(path: impl AsRef<Path>) -> Result<EwwConfig> {
    error_handling_ctx::clear_files();
    EwwConfig::read_from_file(&mut error_handling_ctx::YUCK_FILES.write().unwrap(), path)
}

/// Eww configuration structure.
#[derive(Debug, Clone)]
pub struct EwwConfig {
    widgets: HashMap<String, WidgetDefinition>,
    windows: HashMap<String, EwwWindowDefinition>,
    initial_variables: HashMap<VarName, DynVal>,
    script_vars: HashMap<VarName, ScriptVarDefinition>,
}

impl Default for EwwConfig {
    fn default() -> Self {
        Self { widgets: HashMap::new(), windows: HashMap::new(), initial_variables: HashMap::new(), script_vars: HashMap::new() }
    }
}

impl EwwConfig {
    pub fn read_from_file(files: &mut YuckFiles, path: impl AsRef<Path>) -> Result<Self> {
        if !path.as_ref().exists() {
            bail!("The configuration file `{}` does not exist", path.as_ref().display());
        }
        let config = Config::generate_from_main_file(files, path)?;
        let Config { widget_definitions, window_definitions, var_definitions, mut script_vars } = config;
        script_vars.extend(crate::config::inbuilt::get_inbuilt_vars());
        Ok(EwwConfig {
            windows: window_definitions
                .into_iter()
                .map(|(name, window)| Ok((name, EwwWindowDefinition::generate(&widget_definitions, window)?)))
                .collect::<Result<HashMap<_, _>>>()?,
            widgets: widget_definitions,
            initial_variables: var_definitions.into_iter().map(|(k, v)| (k, v.initial_value)).collect(),
            script_vars,
        })
    }

    // TODO this is kinda ugly
    pub fn generate_initial_state(&self) -> Result<HashMap<VarName, DynVal>> {
        let mut vars = self
            .script_vars
            .iter()
            .map(|(name, var)| Ok((name.clone(), script_var::initial_value(var)?)))
            .collect::<Result<HashMap<_, _>>>()?;
        vars.extend(self.initial_variables.clone());
        Ok(vars)
    }

    pub fn get_windows(&self) -> &HashMap<String, EwwWindowDefinition> {
        &self.windows
    }

    pub fn get_window(&self, name: &String) -> Result<&EwwWindowDefinition> {
        self.windows.get(name).with_context(|| {
            format!(
                "No window named '{}' exists in config.\nThis may also be caused by your config failing to load properly, \
                 please check for any other errors in that case.",
                name
            )
        })
    }

    pub fn get_script_var(&self, name: &VarName) -> Result<&ScriptVarDefinition> {
        self.script_vars.get(name).with_context(|| format!("No script var named '{}' exists", name))
    }

    pub fn get_widget_definitions(&self) -> &HashMap<String, WidgetDefinition> {
        &self.widgets
    }
}
