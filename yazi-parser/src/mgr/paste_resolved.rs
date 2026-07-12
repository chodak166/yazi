use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::ActionCow;
use yazi_shared::url::UrlBuf;

#[derive(Debug, Deserialize)]
pub struct PasteResolvedForm {
	#[serde(default)]
	pub cut:    bool,
	#[serde(default)]
	pub follow: bool,
	#[serde(default)]
	pub items:  Vec<PasteItem>,
}

#[derive(Debug, Deserialize)]
pub struct PasteItem {
	pub from:       UrlBuf,
	pub to:         UrlBuf,
	#[serde(default)]
	pub overwrite:  bool,
	#[serde(default)]
	pub replace:    bool,
}

impl TryFrom<ActionCow> for PasteResolvedForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for PasteResolvedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PasteResolvedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

#[cfg(test)]
mod tests {
	use super::*;
	use hashbrown::HashMap;
	use yazi_shared::{Layer, Source, data::{Data, DataKey}, event::Action, url::UrlLike};

	fn make_action() -> Action {
		yazi_shared::init_tests();
		Action::new("paste_resolved", Source::Emit, Layer::Mgr).unwrap()
	}

	#[test]
	fn test_parse_empty() {
		let action = make_action();
		let form = PasteResolvedForm::try_from(ActionCow::Owned(action)).unwrap();
		assert!(!form.cut);
		assert!(!form.follow);
		assert!(form.items.is_empty());
	}

	#[test]
	fn test_parse_basic() {
		let action = make_action()
			.with("cut", Data::Boolean(true))
			.with("follow", Data::Boolean(false))
			.with("items", Data::List(vec![
				Data::Dict(HashMap::from([
					(DataKey::String("from".into()), Data::String("/src/a.txt".into())),
					(DataKey::String("to".into()), Data::String("/dest/a.txt".into())),
					(DataKey::String("overwrite".into()), Data::Boolean(true)),
				])),
				Data::Dict(HashMap::from([
					(DataKey::String("from".into()), Data::String("/src/b.txt".into())),
					(DataKey::String("to".into()), Data::String("/dest/b_renamed.txt".into())),
					(DataKey::String("overwrite".into()), Data::Boolean(false)),
				])),
			]));

		let form = PasteResolvedForm::try_from(ActionCow::Owned(action)).unwrap();
		assert!(form.cut);
		assert!(!form.follow);
		assert_eq!(form.items.len(), 2);

		assert_eq!(form.items[0].from.name().unwrap(), "a.txt");
		assert_eq!(form.items[0].to.name().unwrap(), "a.txt");
		assert!(form.items[0].overwrite);
		assert!(!form.items[0].replace);

		assert_eq!(form.items[1].from.name().unwrap(), "b.txt");
		assert_eq!(form.items[1].to.name().unwrap(), "b_renamed.txt");
		assert!(!form.items[1].overwrite);
		assert!(!form.items[1].replace);
	}

	#[test]
	fn test_parse_defaults() {
		let action = make_action().with("items", Data::List(vec![
			Data::Dict(HashMap::from([
				(DataKey::String("from".into()), Data::String("/src/c.txt".into())),
				(DataKey::String("to".into()), Data::String("/dest/c.txt".into())),
			])),
		]));

		let form = PasteResolvedForm::try_from(ActionCow::Owned(action)).unwrap();
		assert!(!form.cut);
		assert!(!form.follow);
		assert_eq!(form.items.len(), 1);
		assert!(!form.items[0].overwrite);
		assert!(!form.items[0].replace);
	}

	#[test]
	fn test_parse_replace() {
		let action = make_action().with("items", Data::List(vec![
			Data::Dict(HashMap::from([
				(DataKey::String("from".into()), Data::String("/src/dir".into())),
				(DataKey::String("to".into()), Data::String("/dest/dir".into())),
				(DataKey::String("overwrite".into()), Data::Boolean(true)),
				(DataKey::String("replace".into()), Data::Boolean(true)),
			])),
			Data::Dict(HashMap::from([
				(DataKey::String("from".into()), Data::String("/src/dir2".into())),
				(DataKey::String("to".into()), Data::String("/dest/dir2".into())),
				(DataKey::String("overwrite".into()), Data::Boolean(true)),
				(DataKey::String("replace".into()), Data::Boolean(false)),
			])),
		]));

		let form = PasteResolvedForm::try_from(ActionCow::Owned(action)).unwrap();
		assert_eq!(form.items.len(), 2);

		// Replace mode: destination deleted before copy
		assert!(form.items[0].overwrite);
		assert!(form.items[0].replace);

		// Merge mode: destination merged (overwrite but not replace)
		assert!(form.items[1].overwrite);
		assert!(!form.items[1].replace);
	}

	#[test]
	fn test_parse_mixed_items() {
		// Simulates a full paste_resolved payload with merge, replace, skip, and rename items
		let action = make_action()
			.with("cut", Data::Boolean(false))
			.with("follow", Data::Boolean(false))
			.with("items", Data::List(vec![
				// Non-conflicting: silent paste
				Data::Dict(HashMap::from([
					(DataKey::String("from".into()), Data::String("/src/clean.txt".into())),
					(DataKey::String("to".into()), Data::String("/dest/clean.txt".into())),
					(DataKey::String("overwrite".into()), Data::Boolean(false)),
					(DataKey::String("replace".into()), Data::Boolean(false)),
				])),
				// Merge: overwrite existing
				Data::Dict(HashMap::from([
					(DataKey::String("from".into()), Data::String("/src/merge_dir".into())),
					(DataKey::String("to".into()), Data::String("/dest/merge_dir".into())),
					(DataKey::String("overwrite".into()), Data::Boolean(true)),
					(DataKey::String("replace".into()), Data::Boolean(false)),
				])),
				// Replace: delete destination first
				Data::Dict(HashMap::from([
					(DataKey::String("from".into()), Data::String("/src/replace_dir".into())),
					(DataKey::String("to".into()), Data::String("/dest/replace_dir".into())),
					(DataKey::String("overwrite".into()), Data::Boolean(true)),
					(DataKey::String("replace".into()), Data::Boolean(true)),
				])),
				// Renamed: new non-conflicting destination
				Data::Dict(HashMap::from([
					(DataKey::String("from".into()), Data::String("/src/conflict.txt".into())),
					(DataKey::String("to".into()), Data::String("/dest/conflict-260707.txt".into())),
					(DataKey::String("overwrite".into()), Data::Boolean(false)),
					(DataKey::String("replace".into()), Data::Boolean(false)),
				])),
			]));

		let form = PasteResolvedForm::try_from(ActionCow::Owned(action)).unwrap();
		assert!(!form.cut);
		assert_eq!(form.items.len(), 4);

		// clean
		assert!(!form.items[0].overwrite);
		assert!(!form.items[0].replace);

		// merge
		assert!(form.items[1].overwrite);
		assert!(!form.items[1].replace);

		// replace
		assert!(form.items[2].overwrite);
		assert!(form.items[2].replace);

		// renamed
		assert!(!form.items[3].overwrite);
		assert!(!form.items[3].replace);
		assert_eq!(form.items[3].to.name().unwrap(), "conflict-260707.txt");
	}
}
