//! World generator presets, village structure variants, and biome definitions (`Biome`).
//! Provides color tint filters and snow/precipitation flags without platform branching.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum WorldGeneratorType {
    #[default]
    Default,
    Superflat,
}

pub type WorldType = WorldGeneratorType;

impl WorldGeneratorType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Superflat => "SUPERFLAT",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VillageComponent {
    CenterWell,
    Cottage,
    Blacksmith,
    Farm,
    Library,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    DeepOcean, Ocean, SnowyTaiga, SnowyTundra, AutumnForest, Forest, CherryGrove,
    Plains, Swamp, Savanna, Desert, Mountain, Jungle, Badlands, DarkForest,
    Taiga, BirchForest, IceSpikes, MushroomIsland, Meadow, JaggedPeaks,
    FrozenPeaks, SnowySlopes, River, FrozenRiver, Beach, ColdBeach, StoneBeach,
    SunflowerPlains, FlowerForest, MegaTaiga, JungleEdge, SavannaPlateau,
    Mesa, MesaPlateau, MesaPlateauForest, MesaBryce,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Self::DeepOcean => "DEEP OCEAN", Self::Ocean => "OCEAN", Self::MushroomIsland => "MUSHROOM ISLAND",
            Self::IceSpikes => "ICE SPIKES", Self::SnowyTaiga => "SNOWY TAIGA", Self::SnowyTundra => "SNOWY TUNDRA",
            Self::Taiga => "TAIGA", Self::Desert => "DESERT", Self::Badlands => "BADLANDS",
            Self::Savanna => "SAVANNA", Self::Jungle => "JUNGLE", Self::Swamp => "SWAMP",
            Self::Meadow => "MEADOW", Self::Plains => "PLAINS", Self::BirchForest => "BIRCH FOREST",
            Self::Forest => "FOREST", Self::DarkForest => "DARK FOREST", Self::AutumnForest => "AUTUMN FOREST",
            Self::CherryGrove => "CHERRY GROVE", Self::Mountain => "MOUNTAINS", Self::JaggedPeaks => "JAGGED PEAKS",
            Self::FrozenPeaks => "FROZEN PEAKS", Self::SnowySlopes => "SNOWY SLOPES",
            Self::River => "RIVER", Self::FrozenRiver => "FROZEN RIVER",
            Self::Beach => "BEACH", Self::ColdBeach => "COLD BEACH", Self::StoneBeach => "STONE BEACH",
            Self::SunflowerPlains => "SUNFLOWER PLAINS", Self::FlowerForest => "FLOWER FOREST",
            Self::MegaTaiga => "MEGA TAIGA", Self::JungleEdge => "JUNGLE EDGE",
            Self::SavannaPlateau => "SAVANNA PLATEAU", Self::Mesa => "MESA", Self::MesaPlateau => "MESA PLATEAU",
            Self::MesaPlateauForest => "MESA PLATEAU FOREST", Self::MesaBryce => "MESA BRYCE",
        }
    }

    #[inline]
    pub fn color_filter(self) -> [f32; 3] {
        match self {
            Self::DeepOcean => [0.88, 0.97, 1.12], Self::Ocean => [0.92, 1.02, 1.08],
            Self::SnowyTaiga => [0.92, 0.98, 1.08], Self::SnowyTundra => [0.93, 1.00, 1.11],
            Self::AutumnForest => [1.11, 1.01, 0.89], Self::Forest => [0.97, 1.04, 0.97],
            Self::CherryGrove => [1.08, 0.97, 1.04], Self::Plains => [1.01, 1.03, 0.98],
            Self::Swamp => [0.94, 1.03, 0.86], Self::Savanna => [1.07, 1.04, 0.91],
            Self::Desert => [1.10, 1.03, 0.89], Self::Mountain => [0.97, 1.00, 1.04],
            Self::Jungle => [0.95, 1.08, 0.94], Self::Badlands => [1.12, 0.98, 0.90],
            Self::DarkForest => [0.91, 0.96, 0.93], Self::Taiga => [0.96, 1.02, 1.02],
            Self::BirchForest => [1.02, 1.06, 0.99], Self::IceSpikes => [0.90, 1.03, 1.14],
            Self::MushroomIsland => [1.05, 0.96, 1.08], Self::Meadow => [1.04, 1.05, 0.98],
            Self::JaggedPeaks => [0.95, 0.97, 1.05], Self::FrozenPeaks => [0.89, 1.01, 1.13],
            Self::SnowySlopes => [0.95, 1.01, 1.08],
            Self::River => [0.86, 1.02, 1.12], Self::FrozenRiver => [0.88, 1.03, 1.15],
            Self::Beach => [1.08, 1.06, 0.94], Self::ColdBeach => [0.93, 1.01, 1.10],
            Self::StoneBeach => [0.99, 1.00, 1.04], Self::SunflowerPlains => [1.08, 1.08, 0.91],
            Self::FlowerForest => [1.07, 1.06, 1.00], Self::MegaTaiga => [0.95, 1.00, 0.98],
            Self::JungleEdge => [0.98, 1.06, 0.95], Self::SavannaPlateau => [1.08, 1.03, 0.89],
            Self::Mesa => [1.11, 0.98, 0.89], Self::MesaPlateau => [1.12, 0.99, 0.91], Self::MesaPlateauForest => [1.09, 0.99, 0.92],
            Self::MesaBryce => [1.14, 0.96, 0.87],
        }
    }

    #[inline]
    pub fn has_snow(self) -> bool {
        matches!(self, Self::SnowyTaiga | Self::SnowyTundra | Self::IceSpikes | Self::FrozenPeaks | Self::SnowySlopes | Self::JaggedPeaks | Self::FrozenRiver | Self::ColdBeach)
    }

    #[inline]
    pub fn has_precipitation(self) -> bool {
        !matches!(self, Self::Desert | Self::Badlands | Self::Savanna | Self::SavannaPlateau | Self::Mesa | Self::MesaPlateau | Self::MesaPlateauForest | Self::MesaBryce)
    }
}

