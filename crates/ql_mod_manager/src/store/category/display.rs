use std::fmt::Display;

use crate::store::category::{
    ModCategory, ModCategoryCommon, ModCategoryCurseforge, ModCategoryCurseforgeTechnology,
    ModCategoryCurseforgeWorldgen, ModCategoryModrinth,
};

impl Display for ModCategoryCommon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Utility => "Utility",
            Self::Food => "Food",
            Self::Library => "Library",
            Self::Storage => "Storage",
            Self::Decoration => "Decoration",
            Self::Optimization => "Optimization",
            Self::Magic => "Magic",
            Self::Equipment => "Equipment",
            Self::Adventure => "Adventure",
            Self::Social => "Social",
            Self::Technology => "Technology",
            Self::WorldGen => "World Generation",
        })
    }
}

impl Display for ModCategoryModrinth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Transportation => "Transportation",
            Self::Mobs => "Mobs",
            Self::Management => "Management",
            Self::Minigame => "Minigame",
            Self::GameMechanics => "Game Mechanics",
            Self::Economy => "Economy",
            Self::Cursed => "Cursed",
        })
    }
}

impl Display for ModCategoryCurseforge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ServerUtility => "Server Utility",
            Self::MapInformation => "Map and Information",
            Self::BugFixes => "Bug Fixes",
            Self::Education => "Education",
            Self::Redstone => "Redstone",
            Self::McMiscellaneous => "Miscellaneous",
            Self::CreativeMode => "Creative Mode",
        })
    }
}

impl Display for ModCategoryCurseforgeTechnology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Genetics => "Genetics",
            Self::Energy => "Energy",
            Self::Processing => "Processing",
            Self::Automation => "Automation",
            Self::PlayerTransport => "Player Transport",
            Self::ItemFluidEnergyTransport => "Item, Fluid, and Energy Transport",
        })
    }
}

impl Display for ModCategoryCurseforgeWorldgen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Dimensions => "Dimensions",
            Self::Biomes => "Biomes",
            Self::Structures => "Structures",
            Self::OresResources => "Ores and Resources",
            Self::Mobs => "Mobs",
        })
    }
}

impl Display for ModCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Common(common) => return common.fmt(f),
            Self::Modrinth(modrinth) => return modrinth.fmt(f),
            Self::Curseforge(curseforge) => return curseforge.fmt(f),
            Self::CurseforgeTechnology(ctech) => return ctech.fmt(f),
            Self::CurseforgeWorldgen(cworldgen) => return cworldgen.fmt(f),
        }
    }
}
