use crate::plugins::{
    api::{
        CommandAvailability, HostEvent, Plugin, PluginCommandContribution, PluginError, PluginId,
        PluginResponse, PluginResult,
    },
    manager::PluginFactory,
};

pub const AMAZON_BUILD_PLUGIN_ID: &str = "amazon-build";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmazonBuildCommand {
    BrazilBuild,
    BrazilBuildRelease,
    BbTest,
    BbcClean,
    BwsClean,
    GitPull,
    GitFetch,
    GitPush,
    Cupdate,
    AddPackage,
    RemovePackage,
    VsChange,
    CrUpdate,
    CrNew,
    CrTargetMainline,
}

impl AmazonBuildCommand {
    pub const ALL: [Self; 15] = [
        Self::BrazilBuild,
        Self::BrazilBuildRelease,
        Self::BbTest,
        Self::BbcClean,
        Self::BwsClean,
        Self::GitPull,
        Self::GitFetch,
        Self::GitPush,
        Self::Cupdate,
        Self::AddPackage,
        Self::RemovePackage,
        Self::VsChange,
        Self::CrUpdate,
        Self::CrNew,
        Self::CrTargetMainline,
    ];

    pub fn section(self) -> &'static str {
        match self {
            Self::BrazilBuild
            | Self::BrazilBuildRelease
            | Self::BbTest
            | Self::BbcClean
            | Self::BwsClean => "BUILD",
            Self::GitPull | Self::GitFetch | Self::GitPush => "GIT",
            Self::Cupdate | Self::AddPackage | Self::RemovePackage | Self::VsChange => "WORKSPACE",
            Self::CrUpdate | Self::CrNew | Self::CrTargetMainline => "CODE REVIEW",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BrazilBuild => "Brazil Build",
            Self::BrazilBuildRelease => "Brazil Build Release",
            Self::BbTest => "BB Test",
            Self::BbcClean => "BBC Clean",
            Self::BwsClean => "BWS Clean",
            Self::GitPull => "Git Pull",
            Self::GitFetch => "Git Fetch",
            Self::GitPush => "Git Push",
            Self::Cupdate => "Cupdate",
            Self::AddPackage => "Add Package",
            Self::RemovePackage => "Remove Package",
            Self::VsChange => "VS Change",
            Self::CrUpdate => "CR Update",
            Self::CrNew => "CR New",
            Self::CrTargetMainline => "CR Target Branch (mainline)",
        }
    }

    pub fn command(self, package: Option<&str>) -> Result<String, String> {
        let command = match self {
            Self::BrazilBuild => "brazil build",
            Self::BrazilBuildRelease => "brazil build release",
            Self::BbTest => "bb test",
            Self::BbcClean => "bbc clean",
            Self::BwsClean => "bws clean",
            Self::GitPull => "git pull",
            Self::GitFetch => "git fetch",
            Self::GitPush => "git push",
            Self::Cupdate => "cupdate",
            Self::VsChange => "vs change",
            Self::CrUpdate => "cr --update",
            Self::CrNew => "cr --new",
            Self::CrTargetMainline => "cr --target-branch mainline",
            Self::AddPackage | Self::RemovePackage => {
                let package = package.ok_or_else(|| "Package name is required.".to_string())?;
                if package.is_empty()
                    || !package.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/')
                    })
                {
                    return Err(
                        "Package names may use letters, digits, '-', '_', '.', and '/'.".into(),
                    );
                }
                return Ok(match self {
                    Self::AddPackage => format!("brazil ws add {package}"),
                    Self::RemovePackage => format!("brazil ws remove {package}"),
                    _ => unreachable!(),
                });
            }
        };
        Ok(command.into())
    }

    pub fn needs_package(self) -> bool {
        matches!(self, Self::AddPackage | Self::RemovePackage)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AmazonBuildState {
    selected: usize,
}
impl AmazonBuildState {
    pub fn selected(&self) -> usize {
        self.selected
    }
    pub fn command(&self) -> AmazonBuildCommand {
        AmazonBuildCommand::ALL[self.selected]
    }
    pub fn move_selection(&mut self, delta: i32) {
        self.selected = if delta < 0 {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(AmazonBuildCommand::ALL.len() - 1)
        };
    }
}

pub struct AmazonBuildPluginFactory {
    id: PluginId,
}
impl Default for AmazonBuildPluginFactory {
    fn default() -> Self {
        Self {
            id: PluginId::new(AMAZON_BUILD_PLUGIN_ID).expect("valid plugin id"),
        }
    }
}
impl PluginFactory for AmazonBuildPluginFactory {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn create(&self) -> Box<dyn Plugin> {
        Box::new(AmazonBuildPlugin {
            id: self.id.clone(),
        })
    }
}
pub struct AmazonBuildPlugin {
    id: PluginId,
}
impl Plugin for AmazonBuildPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn set_enabled(&mut self, _: bool) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }
    fn on_host_event(&mut self, _: &HostEvent) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }
    fn handle_result(&mut self, _: PluginResult) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }
    fn contributions(&self) -> Result<Vec<crate::plugins::api::PluginContribution>, PluginError> {
        Ok(Vec::new())
    }
}
impl AmazonBuildPlugin {
    pub fn open_command(&self) -> PluginCommandContribution {
        PluginCommandContribution {
            id: "plugin.amazon-build.open".into(),
            label: "Amazon Build".into(),
            default_key: Some(crate::input::key::KeyChord::control(
                crate::input::key::KeyCode::Character('b'),
            )),
            availability: CommandAvailability::Enabled,
            priority: 95,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_uses_the_requested_commands_and_rejects_unsafe_package_names() {
        assert_eq!(
            AmazonBuildCommand::BrazilBuildRelease.command(None),
            Ok("brazil build release".into())
        );
        assert_eq!(
            AmazonBuildCommand::CrTargetMainline.command(None),
            Ok("cr --target-branch mainline".into())
        );
        assert_eq!(
            AmazonBuildCommand::AddPackage.command(Some("Example-Package")),
            Ok("brazil ws add Example-Package".into())
        );
        assert!(
            AmazonBuildCommand::RemovePackage
                .command(Some("package; rm -rf /"))
                .is_err()
        );
    }
}
