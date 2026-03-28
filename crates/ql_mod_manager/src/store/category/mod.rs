use ql_core::StoreBackendType as B;

mod display;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ModCategory {
    Common(ModCategoryCommon),
    Modrinth(ModCategoryModrinth),
    Curseforge(ModCategoryCurseforge),
    CurseforgeTechnology(ModCategoryCurseforgeTechnology),
    CurseforgeWorldgen(ModCategoryCurseforgeWorldgen),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ModCategoryCommon {
    Utility,
    Food,
    Library,
    Storage,
    Decoration,
    Optimization,
    Magic,
    Equipment,
    Adventure,
    Social,
    Technology,
    WorldGen,
}

impl ModCategoryCommon {
    pub const ALL: [Self; 12] = [
        Self::Utility,
        Self::Food,
        Self::Library,
        Self::Storage,
        Self::Decoration,
        Self::Optimization,
        Self::Magic,
        Self::Equipment,
        Self::Adventure,
        Self::Social,
        Self::Technology,
        Self::WorldGen,
    ];
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ModCategoryModrinth {
    Transportation,
    Mobs,
    Management,
    Minigame,
    GameMechanics,
    Economy,
    Cursed,
}

impl ModCategoryModrinth {
    pub const ALL: [Self; 7] = [
        Self::Transportation,
        Self::Mobs,
        Self::Management,
        Self::Minigame,
        Self::GameMechanics,
        Self::Economy,
        Self::Cursed,
    ];
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ModCategoryCurseforge {
    ServerUtility,
    MapInformation,
    BugFixes,
    Education,
    Redstone,
    McMiscellaneous,
    CreativeMode,
}

impl ModCategoryCurseforge {
    pub const ALL: [Self; 7] = [
        Self::ServerUtility,
        Self::MapInformation,
        Self::BugFixes,
        Self::Education,
        Self::Redstone,
        Self::McMiscellaneous,
        Self::CreativeMode,
    ];
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ModCategoryCurseforgeTechnology {
    Genetics,
    Energy,
    Processing,
    Automation,
    PlayerTransport,
    ItemFluidEnergyTransport,
}

impl ModCategoryCurseforgeTechnology {
    pub const ALL: [Self; 6] = [
        Self::Genetics,
        Self::Energy,
        Self::Processing,
        Self::Automation,
        Self::PlayerTransport,
        Self::ItemFluidEnergyTransport,
    ];
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ModCategoryCurseforgeWorldgen {
    Dimensions,
    Biomes,
    Structures,
    OresResources,
    Mobs,
}

impl ModCategoryCurseforgeWorldgen {
    pub const ALL: [Self; 5] = [
        Self::Dimensions,
        Self::Biomes,
        Self::Structures,
        Self::OresResources,
        Self::Mobs,
    ];
}

impl ModCategory {
    #[must_use]
    pub fn get_id(self, backend: B) -> Option<&'static str> {
        match (self, backend) {
            (Self::Common(common), _) => Some(common.get_id(backend)),
            (Self::Modrinth(modrinth), B::Modrinth) => Some(modrinth.get_id()),
            (Self::Modrinth(_), _) => None,
            (Self::Curseforge(curseforge), B::Curseforge) => Some(curseforge.get_id()),
            (Self::Curseforge(_), _) => None,
            (Self::CurseforgeTechnology(ctech), B::Curseforge) => Some(ctech.get_id()),
            (Self::CurseforgeTechnology(_), _) => None,
            (Self::CurseforgeWorldgen(worldgen), B::Curseforge) => Some(worldgen.get_id()),
            (Self::CurseforgeWorldgen(_), _) => None,
        }
    }
}

impl ModCategoryCommon {
    #[must_use]
    pub fn get_id(self, backend: B) -> &'static str {
        match (self, backend) {
            (Self::Utility, B::Modrinth) => "utility",
            (Self::Utility, B::Curseforge) => "utility-qol",
            (Self::Food, B::Modrinth) => "food",
            (Self::Food, B::Curseforge) => "mc-food",
            (Self::Library, B::Modrinth) => "library",
            (Self::Library, B::Curseforge) => "library-api",
            (Self::Storage, B::Modrinth | B::Curseforge) => "storage",
            (Self::Decoration, B::Modrinth) => "decoration",
            (Self::Decoration, B::Curseforge) => "cosmetic",
            (Self::Optimization, B::Modrinth) => "optimization",
            (Self::Optimization, B::Curseforge) => "performance",
            (Self::Magic, B::Modrinth | B::Curseforge) => "magic",
            (Self::Equipment, B::Modrinth) => "equipment",
            (Self::Equipment, B::Curseforge) => "armor-weapons-tools",
            (Self::Adventure, B::Modrinth) => "adventure",
            (Self::Adventure, B::Curseforge) => "adventure-rpg",
            (Self::Social, B::Modrinth) => "social",
            (Self::Social, B::Curseforge) => "twitch-integration",
            (Self::Technology, B::Modrinth | B::Curseforge) => "technology",
            (Self::WorldGen, B::Modrinth) => "worldgen",
            (Self::WorldGen, B::Curseforge) => "world-gen",
        }
    }
}

impl ModCategoryModrinth {
    #[must_use]
    pub fn get_id(self) -> &'static str {
        match self {
            Self::Transportation => "transportation",
            Self::Mobs => "mobs",
            Self::Management => "management",
            Self::Minigame => "minigame",
            Self::GameMechanics => "game-mechanics",
            Self::Economy => "economy",
            Self::Cursed => "cursed",
        }
    }
}

impl ModCategoryCurseforge {
    #[must_use]
    pub fn get_id(self) -> &'static str {
        match self {
            Self::ServerUtility => "server-utility",
            Self::MapInformation => "map-information",
            Self::BugFixes => "bug-fixes",
            Self::Education => "education",
            Self::Redstone => "redstone",
            Self::McMiscellaneous => "mc-miscellaneous",
            Self::CreativeMode => "creativemode",
        }
    }
}

impl ModCategoryCurseforgeTechnology {
    #[must_use]
    pub fn get_id(self) -> &'static str {
        match self {
            Self::Genetics => "technology-genetics",
            Self::Energy => "technology-energy",
            Self::Processing => "technology-processing",
            Self::Automation => "technology-automation",
            Self::PlayerTransport => "technology-player-transport",
            Self::ItemFluidEnergyTransport => "technology-item-fluid-energy-transport",
        }
    }
}

impl ModCategoryCurseforgeWorldgen {
    #[must_use]
    pub fn get_id(self) -> &'static str {
        match self {
            Self::Dimensions => "world-dimensions",
            Self::Biomes => "world-biomes",
            Self::Structures => "world-structures",
            Self::OresResources => "world-ores-resources",
            Self::Mobs => "world-mobs",
        }
    }
}
