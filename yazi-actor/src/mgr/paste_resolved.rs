use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::mgr::PasteResolvedForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct PasteResolved;

impl Actor for PasteResolved {
	type Form = PasteResolvedForm;

	const NAME: &str = "paste_resolved";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		for item in &form.items {
			if form.cut {
				cx.core.tasks.file_cut_one(&item.from, &item.to, item.overwrite, item.replace);
			} else {
				cx.core
					.tasks
					.file_copy_one(&item.from, &item.to, item.overwrite, item.replace, form.follow);
			}
		}

		if form.cut {
			let mgr = &mut cx.core.mgr;
			let urls: Vec<_> = form.items.iter().map(|i| i.from.clone()).collect();
			mgr.tabs.iter_mut().for_each(|t| _ = t.selected.remove_many(urls.iter()));
			act!(mgr:unyank, cx)
		} else {
			succ!();
		}
	}
}
