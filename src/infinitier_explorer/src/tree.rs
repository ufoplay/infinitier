//! The resource tree of the left panel: one group per file extension, holding
//! that type's resources, every group closed at first. [`ResourceTree`]
//! keeps the visible rows and the selection; [`TreeModel`] serves those rows
//! to the Slint `ListView`, notifying only the rows that change, so opening
//! a group of thousands of resources doesn't rebuild the list.

use std::cell::RefCell;
use std::collections::BTreeMap;

use infinitier_core::game::{DataOrigin, GameData, ResourceId};
use slint::{Model, ModelNotify, ModelTracker};

use crate::{TreeNav, TreeRow};

/// File extension → (row label → resource index), both sorted.
type Groups = BTreeMap<&'static str, BTreeMap<String, ResourceId>>;

/// Group `game_data`'s resources by extension. A resource from a directory
/// (the override folder) is marked `(O)`.
fn group_resources(game_data: &GameData) -> Groups {
    let mut groups = Groups::new();
    for (index, entry) in game_data.resources().iter().enumerate() {
        let ext = entry.r#type.get_extension().unwrap_or("unknown");
        let label = if matches!(entry.data_origin, DataOrigin::Dir { .. }) {
            format!("{} (O)", entry.resource_name_with_extension())
        } else {
            entry.resource_name_with_extension()
        };
        groups.entry(ext).or_default().insert(label, index);
    }
    groups
}

struct Group {
    /// `"{ext} ({count})"`.
    label: String,
    entries: Vec<(String, ResourceId)>,
}

/// A node of the tree: a group, or the `entry`th resource of a group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Node {
    Group(usize),
    Entry(usize, usize),
}

impl Node {
    fn group(self) -> usize {
        match self {
            Node::Group(group) | Node::Entry(group, _) => group,
        }
    }
}

/// A change to the visible rows, for the model's notifications. Each one is
/// relative to the rows as left by the previous one.
#[derive(Debug, PartialEq, Eq)]
enum RowChange {
    Changed(usize),
    Added { at: usize, count: usize },
    Removed { at: usize, count: usize },
}

/// What a click or a key did.
#[derive(Debug, Default)]
pub struct Outcome {
    changes: Vec<RowChange>,
    /// The resource to show: a resource's row was selected.
    pub open: Option<ResourceId>,
    /// The newly selected row, to scroll into view.
    pub scroll_to: Option<usize>,
}

pub struct ResourceTree {
    groups: Vec<Group>,
    expanded: Vec<bool>,
    /// The visible nodes, in order: every group, followed by its resources
    /// when it is open.
    rows: Vec<Node>,
    selected: Option<Node>,
}

impl ResourceTree {
    pub fn new(game_data: &GameData) -> Self {
        let groups: Vec<Group> = group_resources(game_data)
            .into_iter()
            .map(|(ext, entries)| Group {
                label: format!("{ext} ({})", entries.len()),
                entries: entries.into_iter().collect(),
            })
            .collect();
        Self {
            expanded: vec![false; groups.len()],
            rows: (0..groups.len()).map(Node::Group).collect(),
            groups,
            selected: None,
        }
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row(&self, row: usize) -> Option<TreeRow> {
        let node = *self.rows.get(row)?;
        let selected = self.selected == Some(node);
        Some(match node {
            Node::Group(group) => TreeRow {
                text: self.groups[group].label.as_str().into(),
                level: 0,
                expandable: true,
                expanded: self.expanded[group],
                selected,
            },
            Node::Entry(group, entry) => TreeRow {
                text: self.groups[group].entries[entry].0.as_str().into(),
                level: 1,
                expandable: false,
                expanded: false,
                selected,
            },
        })
    }

    fn row_of(&self, node: Node) -> Option<usize> {
        self.rows.iter().position(|row| *row == node)
    }

    /// The selected node's row; a resource hidden in a closed group stands
    /// for its group's row.
    fn selected_row(&self) -> Option<usize> {
        let node = self.selected?;
        self.row_of(node)
            .or_else(|| self.row_of(Node::Group(node.group())))
    }

    /// Open or close the group on `row`; a resource row does nothing.
    fn toggle(&mut self, row: usize) -> Vec<RowChange> {
        let Some(Node::Group(group)) = self.rows.get(row).copied() else {
            return Vec::new();
        };
        let count = self.groups[group].entries.len();
        let at = row + 1;
        let change = if self.expanded[group] {
            self.rows.drain(at..at + count);
            RowChange::Removed { at, count }
        } else {
            let entries = (0..count).map(|entry| Node::Entry(group, entry));
            self.rows.splice(at..at, entries);
            RowChange::Added { at, count }
        };
        self.expanded[group] = !self.expanded[group];
        vec![change, RowChange::Changed(row)]
    }

    /// Select the node on `row`: a resource is opened.
    fn select(&mut self, row: usize, outcome: &mut Outcome) {
        let Some(node) = self.rows.get(row).copied() else {
            return;
        };
        if let Some(old) = self.selected.and_then(|old| self.row_of(old)) {
            outcome.changes.push(RowChange::Changed(old));
        }
        self.selected = Some(node);
        outcome.changes.push(RowChange::Changed(row));
        outcome.scroll_to = Some(row);
        if let Node::Entry(group, entry) = node {
            outcome.open = Some(self.groups[group].entries[entry].1);
        }
    }

    /// A click on `row`: select it, and open or close a group.
    fn click(&mut self, row: usize) -> Outcome {
        let mut outcome = Outcome::default();
        self.select(row, &mut outcome);
        if matches!(self.rows.get(row), Some(Node::Group(_))) {
            outcome.changes.extend(self.toggle(row));
        }
        outcome
    }

    /// A keyboard move from the selected row.
    fn navigate(&mut self, nav: TreeNav) -> Outcome {
        let mut outcome = Outcome::default();
        let Some(last) = self.rows.len().checked_sub(1) else {
            return outcome;
        };
        let Some(current) = self.selected_row() else {
            self.select(0, &mut outcome);
            return outcome;
        };
        match (nav, self.rows[current]) {
            (TreeNav::Previous, _) => self.select(current.saturating_sub(1), &mut outcome),
            (TreeNav::Next, _) => self.select((current + 1).min(last), &mut outcome),
            (TreeNav::Collapse, Node::Entry(group, _)) => {
                if let Some(row) = self.row_of(Node::Group(group)) {
                    self.select(row, &mut outcome);
                }
            }
            (TreeNav::Collapse, Node::Group(group)) => {
                if self.expanded[group] {
                    outcome.changes.extend(self.toggle(current));
                }
                outcome.scroll_to = Some(current);
            }
            (TreeNav::Expand, Node::Group(group)) => {
                if !self.expanded[group] {
                    outcome.changes.extend(self.toggle(current));
                    outcome.scroll_to = Some(current);
                } else if !self.groups[group].entries.is_empty() {
                    self.select(current + 1, &mut outcome);
                }
            }
            (TreeNav::Expand, Node::Entry(..)) => {}
        }
        outcome
    }
}

/// [`ResourceTree`]'s visible rows as a Slint model.
pub struct TreeModel {
    tree: RefCell<ResourceTree>,
    notify: ModelNotify,
}

impl TreeModel {
    pub fn new(tree: ResourceTree) -> Self {
        Self {
            tree: RefCell::new(tree),
            notify: ModelNotify::default(),
        }
    }

    pub fn click(&self, row: usize) -> Outcome {
        let outcome = self.tree.borrow_mut().click(row);
        self.notify_changes(&outcome.changes);
        outcome
    }

    pub fn toggle(&self, row: usize) {
        let changes = self.tree.borrow_mut().toggle(row);
        self.notify_changes(&changes);
    }

    pub fn navigate(&self, nav: TreeNav) -> Outcome {
        let outcome = self.tree.borrow_mut().navigate(nav);
        self.notify_changes(&outcome.changes);
        outcome
    }

    /// Tell the view what changed. Called with the tree no longer borrowed:
    /// the view reads the changed rows back right away.
    fn notify_changes(&self, changes: &[RowChange]) {
        for change in changes {
            match *change {
                RowChange::Changed(row) => self.notify.row_changed(row),
                RowChange::Added { at, count } => self.notify.row_added(at, count),
                RowChange::Removed { at, count } => self.notify.row_removed(at, count),
            }
        }
    }
}

impl Model for TreeModel {
    type Data = TreeRow;

    fn row_count(&self) -> usize {
        self.tree.borrow().row_count()
    }

    fn row_data(&self, row: usize) -> Option<TreeRow> {
        self.tree.borrow().row(row)
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use infinitier_core::{
        fs::CaseInsensitiveFS,
        game::{GameData, GameResource},
        resource::{Game, key::ResourceType},
    };

    fn make_game_data(entries: Vec<(&str, ResourceType)>) -> GameData {
        make_game_data_with_origins(
            entries
                .into_iter()
                .map(|(name, r#type)| (name, r#type, DataOrigin::Missing))
                .collect(),
        )
    }

    fn make_game_data_with_origins(entries: Vec<(&str, ResourceType, DataOrigin)>) -> GameData {
        GameData::new(
            entries
                .into_iter()
                .map(|(name, r#type, data_origin)| GameResource {
                    game_type: Game::Bg2,
                    name: name.to_string(),
                    r#type,
                    file_size: None,
                    datasource: None,
                    data_origin,
                    imported: None,
                })
                .collect(),
            Game::Bg2,
            CaseInsensitiveFS::empty(),
        )
    }

    /// resources indices:
    ///   0: CCHAN05  Bmp
    ///   1: MINSCM   Bmp
    ///   2: AAATEST  Bmp
    ///   3: SPHEART  Bam
    ///   4: 1CHELM   Bam
    ///   5: AR0072   Wed
    ///   6: UNKNOWN  Unknown(0x9999)
    fn sample_game_data() -> GameData {
        make_game_data(vec![
            ("CCHAN05", ResourceType::Bmp),
            ("MINSCM", ResourceType::Bmp),
            ("AAATEST", ResourceType::Bmp),
            ("SPHEART", ResourceType::Bam),
            ("1CHELM", ResourceType::Bam),
            ("AR0072", ResourceType::Wed),
            ("UNKNOWN", ResourceType::Unknown(0x9999)),
        ])
    }

    fn texts(tree: &ResourceTree) -> Vec<String> {
        (0..tree.row_count())
            .filter_map(|row| tree.row(row))
            .map(|row| row.text.to_string())
            .collect()
    }

    #[test]
    fn test_game_data_tree_view() {
        let groups = group_resources(&sample_game_data());

        // Groups are keyed by extension, sorted alphabetically: bam < bmp < unknown < wed
        assert_eq!(groups.len(), 4);

        let entries = &groups["bam"];
        assert_eq!(entries.len(), 2);
        assert_eq!(entries["1CHELM.bam"], 4);
        assert_eq!(entries["SPHEART.bam"], 3);

        let entries = &groups["bmp"];
        assert_eq!(entries.len(), 3);
        assert_eq!(entries["AAATEST.bmp"], 2);
        assert_eq!(entries["CCHAN05.bmp"], 0);
        assert_eq!(entries["MINSCM.bmp"], 1);

        let entries = &groups["unknown"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries["UNKNOWN"], 6);

        let entries = &groups["wed"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries["AR0072.wed"], 5);
    }

    #[test]
    fn test_override_label() {
        let fs = CaseInsensitiveFS::new("./").expect("the crate directory is readable");
        let path = fs.list_files("", None, false)[0].clone();

        let game_data = make_game_data_with_origins(vec![
            ("CCHAN05", ResourceType::Bmp, DataOrigin::Missing),
            (
                "MINSCM",
                ResourceType::Bmp,
                DataOrigin::Dir {
                    name: "override".to_string(),
                    path,
                },
            ),
        ]);

        let groups = group_resources(&game_data);
        let entries = &groups["bmp"];
        assert_eq!(entries.len(), 2);
        assert_eq!(entries["CCHAN05.bmp"], 0);
        assert_eq!(entries["MINSCM.bmp (O)"], 1);
    }

    #[test]
    fn groups_start_closed() {
        let tree = ResourceTree::new(&sample_game_data());
        assert_eq!(
            texts(&tree),
            ["bam (2)", "bmp (3)", "unknown (1)", "wed (1)"]
        );
        assert!(
            tree.row(0)
                .is_some_and(|row| row.expandable && !row.expanded)
        );
    }

    #[test]
    fn clicking_a_group_opens_and_closes_it() {
        let mut tree = ResourceTree::new(&sample_game_data());

        let outcome = tree.click(1);
        assert_eq!(outcome.open, None);
        assert_eq!(
            outcome.changes,
            [
                RowChange::Changed(1),
                RowChange::Added { at: 2, count: 3 },
                RowChange::Changed(1),
            ]
        );
        assert_eq!(
            texts(&tree),
            [
                "bam (2)",
                "bmp (3)",
                "AAATEST.bmp",
                "CCHAN05.bmp",
                "MINSCM.bmp",
                "unknown (1)",
                "wed (1)"
            ]
        );
        assert!(tree.row(2).is_some_and(|row| row.level == 1));

        let outcome = tree.click(1);
        assert!(
            outcome
                .changes
                .contains(&RowChange::Removed { at: 2, count: 3 })
        );
        assert_eq!(tree.row_count(), 4);
    }

    #[test]
    fn clicking_a_resource_opens_it() {
        let mut tree = ResourceTree::new(&sample_game_data());
        tree.click(0);

        let outcome = tree.click(1);
        assert_eq!(outcome.open, Some(4), "1CHELM.bam sorts first");
        assert!(tree.row(1).is_some_and(|row| row.selected));
        assert!(tree.row(0).is_some_and(|row| !row.selected));
    }

    #[test]
    fn keys_move_through_the_tree() {
        let mut tree = ResourceTree::new(&sample_game_data());

        // Nothing selected: any key selects the first row.
        assert_eq!(tree.navigate(TreeNav::Next).scroll_to, Some(0));
        // Right opens the group, then goes into it.
        tree.navigate(TreeNav::Expand);
        assert_eq!(tree.row_count(), 6);
        let outcome = tree.navigate(TreeNav::Expand);
        assert_eq!(outcome.open, Some(4));
        // Down moves to the next resource and opens it.
        assert_eq!(tree.navigate(TreeNav::Next).open, Some(3));
        // Left goes up to the group, then closes it.
        let outcome = tree.navigate(TreeNav::Collapse);
        assert_eq!((outcome.open, outcome.scroll_to), (None, Some(0)));
        tree.navigate(TreeNav::Collapse);
        assert_eq!(tree.row_count(), 4);
        // Up stops at the top.
        assert_eq!(tree.navigate(TreeNav::Previous).scroll_to, Some(0));
    }

    #[test]
    fn a_resource_hidden_in_a_closed_group_moves_from_the_group() {
        let mut tree = ResourceTree::new(&sample_game_data());
        tree.click(0);
        tree.click(1);
        // Close bam with the mouse: the selected 1CHELM.bam is hidden.
        tree.toggle(0);

        assert_eq!(tree.navigate(TreeNav::Next).scroll_to, Some(1));
    }
}
