use serde::Deserialize;
use std::collections::HashMap;

/// Container struct for all of the [`Db2BattlePetSpecies`] rows.
pub struct Db2BattlePetSpeciesTable {
    // TODO(seputaes) Do we need to keep this data in memory?
    // species: HashMap<u32, Db2BattlePetSpecies>,
    spell_to_species: HashMap<u32, u32>,
}

/// A struct representation of a single row in the BattlePetSpecies DB2 table
/// of World of Warcraft game files.
///
/// This table contains a significant amount of extra data than the fields on this struct,
/// but they're ignored here since those field's aren't used ... yet.
#[derive(Debug, Deserialize)]
pub struct Db2BattlePetSpecies {
    // The unique ID of the Battle Pet Species.
    #[serde(rename = "ID")]
    pub id: u32,

    // The spell that is used to summon the pet.
    // This can be used to reverse-lookup an
    // item on the AH that is a pet but isn't in a pet cage (82800)
    // in order to find its species
    #[serde(rename = "SummonSpellID")]
    pub summon_spell_id: u32,
}

/// Functionality for working with Battle Pet Species and their effect on items.
///
/// In addition to mapping the rows into a [`Db2BattlePetSpecies`],
/// parsing is done which maps all Summon Spell IDs to its associated Species ID
impl Db2BattlePetSpeciesTable {
    pub fn from_csv(csv: &str) -> Db2BattlePetSpeciesTable {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2BattlePetSpecies>();

        let mut spell_to_species = HashMap::new();

        for row in iter {
            // TODO(seputaes): Logging for the error
            if row.is_err() {
                continue;
            }

            let row = row.unwrap();

            // If the summon spell ID is 0, the current theory is that it
            // is not able to be in a pet cage, and thus can't be sold on the AH
            // TODO(seputaes): Maybe have some logging around this to confirm this theory
            //                 using real AH data.
            if row.summon_spell_id == 0 {
                continue;
            }

            if spell_to_species.contains_key(&row.summon_spell_id) {
                // from testing, it looks like if there's a duplicate
                // spell ID, the first one in the table wins
                // as of 9.0.5, duplicate spell IDs are:

                // 15048
                // 25162
                // 53082
                // 89472
                // 132762
                // 135259
                // 138161
                // 149810
                // 170272
                // 291537
                continue;
            }

            spell_to_species.insert(row.summon_spell_id, row.id);
        }

        Db2BattlePetSpeciesTable { spell_to_species }
    }
}

/// Container struct for all of the [`Db2CurvePoints`] rows.
pub struct Db2CurvePoints {
    // TODO(seputaes) Do we need to keep this data in memory?
    // points: Vec<DB2CurvePoint>,
    /// Mapping of Curve IDs to all of the curve `(x, y)` values.
    curve_ids: HashMap<u32, Vec<(f64, f64)>>,
}

/// A struct representation of a single row in the CurvePoint DB2 table
/// of World of Warcraft game files.
#[derive(Debug, Deserialize)]
pub struct Db2CurvePoint {
    /// The ID of a single Curve Point `(x, y)` coordinate.
    #[serde(rename = "ID")]
    pub id: u32,

    /// The `x` coordinate, which corresponds to Player Level
    #[serde(rename = "Pos[0]")]
    pub x: f64,

    /// The `y` coordinate, which corresponds to Item Level
    #[serde(rename = "Pos[1]")]
    pub y: f64,

    /// The `x` coordinate prior to the Patch 9.0 Item Level squish.
    #[serde(rename = "PosPreSquish[0]")]
    pub x_pre_squish: f64,

    /// The `y` coordinate prior to the Patch 9.0 Item Level squish.
    #[serde(rename = "PosPreSquish[1]")]
    pub y_pre_squish: f64,

    /// The ID of the overall curve, which is made up of one or more
    /// curve points. This ID is tied to Bonus IDs via the
    /// [`DB2BItemBonuses`] table.
    #[serde(rename = "CurveID")]
    pub curve_id: u32,

    /// The order index within the table if between "like" items.
    /// Typically unused in the context of Ethereal Post.
    #[serde(rename = "OrderIndex")]
    pub order_index: u16,
}

/// Functionality for working with Curve Points and their effect on items.
///
/// In addition to mapping the rows into a [`Db2CurvePoint`],
/// parsing is done which maps all Curve IDs to all `(x, y)` coordinates
/// associated with that ID for fast retrieval.
impl Db2CurvePoints {
    /// Deserializes a CSV string which represents the DB2 CurvePoints table
    /// in World of Warcraft.
    pub fn from_csv(csv: &str) -> Db2CurvePoints {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2CurvePoint>();

        // TODO(seputaes): Some rows contain floats for their `x` and `y` coordinates
        // I have no idea if they're ever used for our context, but for now
        // we're just casting them into u32 which is _not_ safe.
        let mut curve_ids: HashMap<u32, Vec<(f64, f64)>> = HashMap::new();

        for point in iter {
            if point.is_err() {
                continue;
            }
            let point = point.unwrap();

            curve_ids
                .entry(point.curve_id)
                .or_insert_with(Vec::new)
                .push((point.x, point.y));
        }

        Db2CurvePoints {
            // TODO(seputaes) Do we need to keep this data in memory?
            // points,
            curve_ids,
        }
    }

    /// Find the curve `(x, y)` coordinates associated with a Curve ID,
    /// if it exists.
    pub fn points(&self, curve_id: &u32) -> Option<&Vec<(f64, f64)>> {
        self.curve_ids.get(curve_id)
    }
}

/// Container struct for all of the [`Db2ItemBonus`] rows.
pub struct Db2ItemBonuses {
    // TODO(seputaes) Do we need to keep this data in memory?
    // bonuses: Vec<DB2ItemBonus>,

    // Mapping of Bonus IDs to Curve IDs (for bonuses which are curves).
    curve_ids: HashMap<u32, u32>,

    // Mapping of Bonus IDs to Item Level Adjustments (for bonuses which are adjustments).
    ilvl_adjustments: HashMap<u32, i32>,
}

/// A struct representation of a single row in the ItemBonus DB2 table of
/// World of Warcraft game files.
#[derive(Debug, Deserialize)]
pub struct Db2ItemBonus {
    /// The ID of the Item Bonus
    #[serde(rename = "ID")]
    pub id: u32,

    /// The first value associated with the item bonus. In the context of
    /// auctions, this will typically the item level adjustment
    /// which should be applied to an item's base item level.
    #[serde(rename = "Value[0]")]
    pub value0: i32,

    /// The second value associated with the item bonus. Not used
    /// in the context of Ethereal Post.
    #[serde(rename = "Value[1]")]
    pub value1: i32,

    /// The third value associated with the item bonus. Not used
    /// in the context of Ethereal Post.
    #[serde(rename = "Value[2]")]
    pub value2: i32,

    /// The forth value associated with the item bonus. In the context of
    /// auctions, this will typically contain a Curve ID for the bonus
    /// which can be further looked up in the [`Db2CurvePoint`] struct.
    #[serde(rename = "Value[3]")]
    pub value3: i32,

    /// The Parent Bonus List ID for the Bonus. This is the number
    /// that will appear in an Item's `bonusList` field.
    #[serde(rename = "ParentItemBonusListID")]
    pub parent_item_bonus_list_id: u32,

    /// The type of the bonus. In the context of auctions, types
    /// `1`, `11`, and `13` are the ones that matter most since these
    /// impact the final level of the item.
    ///
    /// * `1` - A simple item level adjustment. The adjustment value
    ///   will be contained in the `value0` field.
    /// * `11` - A scaling distribution, which implies there is a "Curve"
    ///   associated with this bonus. The Curve ID will be in the `value3`
    ///   field.
    /// * `13` - A fixed scaling distribution, which implies there is a "Curve"
    ///   associated with this bonus. The Curve ID will be in the `value3`
    ///   field.
    #[serde(rename = "Type")]
    pub type_id: u16,

    /// The order index within the table if between "like" items.
    /// Typically unused in the context of Ethereal Post.
    #[serde(rename = "OrderIndex")]
    pub order_index: u16,
}

/// Functionality for working with item bonuses an their effect on items.
///
/// In addition to mapping the rows into a [`Db2ItemBonus`],
/// parsing is done which maps all Bonus IDs (ParentItemLevelBonus)
/// to their corresponding Curve ID and Item Level Adjustment values.
///
/// This these IDs and adjustments can be gathered by using the
/// associated [resolve_ilvl_adjustment](#method.resolve_ilvl_adjustment)
/// and [resolve_curve_id](#method.resolve_curve_id) methods for a
/// given set of bonus IDs on an item.
///
/// # Special Considerations
///
/// An [Item](`crate::battlenet::auctions::Item`) can have multiple
/// bonus IDs which map to both multiple item level adjustment values
/// as well as multiple curves. In these cases, the following priority
/// is used for determining which should be used:
///
/// 1. If there is exactly 1 curve ID bonus, that curve is used and is
///    applied to the player's level when the item dropped.
/// 2. If there is more than 1 curve ID bonus, the _max_ curve ***ID***
///    is used and is applied to the player's level when the item dropped.
/// 3. If there is exactly 1 item level adjustment bonus, that adjustment is applied
///    to the item's base level.
/// 4. If there is more than 1 item level adjustment bonus, the sum of the
///    adjustments is applied to the item's base level.
/// 5. If there are no level adjustment or curve bonuses, the item's base
///    level is used.
///
/// # Examples
///
/// Consider the following scenarios:
///
/// ## Bonus IDs: `[6885, 6908]`
///
/// Both of these bonus IDs map to curves (`19995` and `17967`, respectively).
/// Because there are **2** curves tied to this item, the second rule above applies,
/// and curve ID `19995` is used.
///
/// ## Bonus IDs: `[1520, 5852]`
///
/// Both of these bonus IDs map to level adjustments (`48` and `7`, respectively).
/// Because there are **2** curves tied to this item, the fourth rule above applies
/// and the sum `55` is the resulting level adjustemtn.
///
/// ## Bonus IDs: `[6908, 1520]`
///
/// `6908` maps to a curve ID, and `1520` maps to a level adjustment.
/// Because there is _exactly 1_ curve tied to this item, the first rule above applies
/// and the curve ID associated with bonus ID `6908` is to be used.
impl Db2ItemBonuses {
    /// Deserializes a CSV string which represents the DB2 ItemBonus table
    /// in World of Warcraft.
    pub fn from_csv(csv: &str) -> Db2ItemBonuses {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2ItemBonus>();

        let mut curve_ids: HashMap<u32, u32> = HashMap::new();
        let mut ilvl_adjustments: HashMap<u32, i32> = HashMap::new();

        for bonus in iter {
            if bonus.is_err() {
                continue;
            }
            let bonus = bonus.unwrap();

            // Map the associated curve ids or item level adjustments
            match bonus.type_id {
                // item level adjustment (ItemLevel)
                1 => {
                    ilvl_adjustments.insert(bonus.parent_item_bonus_list_id, bonus.value0);
                }
                // Curve adjustment types (ScalingStatDistributionFixed or ScalingStatDistribution)
                13 | 11 => {
                    curve_ids.insert(bonus.parent_item_bonus_list_id, bonus.value3 as u32);
                }
                _ => {}
            }
        }

        Db2ItemBonuses {
            // TODO(seputaes) Do we need to keep this data in memory? // bonuses,
            curve_ids,
            ilvl_adjustments,
        }
    }

    /// Finds the simple item level adjustment associated with a Bonus ID, if one exists.
    ///
    /// If there are multiple bonus IDs on an item, you should use
    /// [resolve_ilvl_adjustment](#method.resolve_ilvl_adjustment) instead.
    pub fn ilvl_adjustment(&self, bonus_id: &u32) -> Option<i32> {
        self.ilvl_adjustments.get(bonus_id).copied()
    }

    /// Resolves the item level adjustment that should be applied to an
    /// item's base level based on its bonus IDs.
    ///
    /// An [Item](`crate::battlenet:auctions::Item`) can have multiple
    /// bonus IDs on it which apply a level adjustment to the item's base
    /// item level. There can be more than one, in which case the sum
    /// of the adjustments is applied to the items' base level to
    /// determine the final item level.
    ///
    /// If an item has _both_ one or more simple level adjustments
    /// and a bonus which applies a Curve, the curve will take
    /// precedence.
    ///
    /// See [resolve_curve_id](#method.resolve_curve_id).
    pub fn resolve_ilvl_adjustment(&self, bonus_ids: &[u32]) -> Option<i32> {
        let mut adjustment: Option<i32> = None;

        for bonus_id in bonus_ids {
            let bonus_diff = self.ilvl_adjustments.get(bonus_id);
            if let Some(diff) = bonus_diff {
                adjustment = adjustment.or(Some(0)).map(|a| a + diff);
            }
        }

        adjustment
    }

    /// Finds the Curve ID associated with a Bonus ID, if one exists.
    ///
    /// If there are multiple bonus IDs on an item, you should use
    /// [resolve_curve_id](#method.resolve_curve_id) instead.
    pub fn curve_id(&self, bonus_id: &u32) -> Option<u32> {
        self.curve_ids.get(bonus_id).copied()
    }

    /// Resolve the Curve ID that should be used for calculating an item's level.
    ///
    /// Some items can have multiple bonuses that map to a Curve ID. Why this is the
    /// case is not immediately clear, but it is consistent that in such cases the
    /// highest curve ID takes precedence.
    ///
    /// This method iterates over the `bonus_id` and returns the
    /// curve ID that should be used to calculate the item's level, if
    /// applicable.
    ///
    /// # Arguments
    ///
    /// * `bonus_ids` - The bonus IDs present on the auction
    ///   [Item](`crate::battlenet:auctions::Item`).
    pub fn resolve_curve_id(&self, bonus_ids: &[u32]) -> Option<u32> {
        let mut highest = 0;

        for bonus_id in bonus_ids {
            let curve_id = self.curve_id(bonus_id);
            if let Some(curve_id) = curve_id {
                if curve_id > highest {
                    highest = curve_id;
                }
            }
        }

        match highest {
            0 => None,
            _ => Some(highest),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BATTLE_PET_SPECIES_CSV_HEADER: &str =
        "Description_lang,SourceText_lang,ID,CreatureID,SummonSpellID,IconFileDataID,PetTypeEnum,Flags,SourceTypeEnum,CardUIModelSceneID,LoadoutUIModelSceneID,CovenantID";
    const CURVE_CSV_HEADER: &str =
        "ID,Pos[0],Pos[1],PosPreSquish[0],PosPreSquish[1],CurveID,OrderIndex";
    const ITEM_BONUSES_CSV_HEADER: &str =
        "ID,Value[0],Value[1],Value[2],Value[3],ParentItemBonusListID,Type,OrderIndex";

    #[test]
    fn battle_pet_species_uses_first_spell_id() {
        let mut csv = String::from(BATTLE_PET_SPECIES_CSV_HEADER);
        csv.push_str("\n\"Possibly explosive, definitely adorable. Keep away from open flame.\",|cFFFFD200Profession: |rEngineering,85,9656,15048,133712,9,2,3,6,7,0");
        csv.push_str("\n\"The first bombling created in the Underhold, Siegecrafter Blackfuse couldn't bear to see it destroyed, and kept it as a friendly, if explosive, pet.\",|cFFFFD200Drop:|r Siegecrafter Blackfuse|n|cFFFFD200Raid:|r Siege of Orgrimmar,1322,73352,15048,897633,9,2,0,6,7,0");

        let table = Db2BattlePetSpeciesTable::from_csv(&csv);
        assert_eq!(85, *table.spell_to_species.get(&15048).unwrap());
    }

    #[test]
    fn curve_single_curve() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert_eq!(vec![(1.0f64, 6.0f64)], *table.points(&5).unwrap());
    }

    #[test]
    fn curve_multiple_curves() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0\n9,25,31,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert_eq!(vec![(1.0, 6.0), (25.0, 31.0)], *table.points(&5).unwrap());
    }

    #[test]
    fn curve_mixed_curves() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0\n2,3,4,0,1,9,0\n9,25,31,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert_eq!(vec![(1.0, 6.0), (25.0, 31.0)], *table.points(&5).unwrap());
    }

    #[test]
    fn curve_no_curve_with_id() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0\n2,3,4,0,1,9,0\n9,25,31,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert!(table.points(&12).is_none());
    }

    #[test]
    fn resolve_ilvl_adjustment_single_adjustment() {
        let mut csv = String::from(ITEM_BONUSES_CSV_HEADER);
        csv.push_str("\n5,-2,0,0,0,58,1,0");

        let table = Db2ItemBonuses::from_csv(&csv);
        assert_eq!(-2, table.resolve_ilvl_adjustment(&[58]).unwrap());
    }

    #[test]
    fn resolve_ilvl_adjustment_multiple_adjustments() {
        let mut csv = String::from(ITEM_BONUSES_CSV_HEADER);
        csv.push_str("\n5,-2,0,0,0,58,1,0\n9,40,0,0,0,72,1,0");

        let table = Db2ItemBonuses::from_csv(&csv);
        assert_eq!(38, table.resolve_ilvl_adjustment(&[58, 72]).unwrap());
    }

    #[test]
    fn resolve_ilvl_adjustment_multiple_adjustments_mixed_types() {
        let mut csv = String::from(ITEM_BONUSES_CSV_HEADER);
        csv.push_str("\n5,-2,0,0,0,58,1,0\n9,40,0,0,0,72,1,0\n3,0,0,0,1222,72,11,0");

        let table = Db2ItemBonuses::from_csv(&csv);
        assert_eq!(38, table.resolve_ilvl_adjustment(&[58, 72]).unwrap());
    }

    #[test]
    fn resolve_curve_id_single_curve() {
        let mut csv = String::from(ITEM_BONUSES_CSV_HEADER);
        csv.push_str("\n5,0,0,0,19932,58,11,0");

        let table = Db2ItemBonuses::from_csv(&csv);
        assert_eq!(19932, table.resolve_curve_id(&[58, 72]).unwrap());
    }

    #[test]
    fn resolve_curve_id_multiple_curves() {
        let mut csv = String::from(ITEM_BONUSES_CSV_HEADER);
        csv.push_str("\n5,0,0,0,19932,58,11,0\n9,0,0,0,17322,72,11,0");

        let table = Db2ItemBonuses::from_csv(&csv);
        assert_eq!(19932, table.resolve_curve_id(&[58, 72]).unwrap());
    }

    #[test]
    fn resolve_curve_id_fixed_type() {
        let mut csv = String::from(ITEM_BONUSES_CSV_HEADER);
        csv.push_str("\n5,0,0,0,17322,58,13,0\n9,0,0,0,19932,72,13,0");

        let table = Db2ItemBonuses::from_csv(&csv);
        assert_eq!(19932, table.resolve_curve_id(&[58, 72]).unwrap());
    }
}
