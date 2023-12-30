use floem::{
	event::{Event, EventListener},
	id::Id,
	keyboard::{KeyCode, PhysicalKey},
	kurbo::Size,
	reactive::{create_rw_signal, RwSignal, WriteSignal},
	style::{AlignContent, AlignItems, CursorStyle, Display, Position},
	view::View,
	views::{container, h_stack, label, static_list, svg, v_stack, Decorators},
	Clipboard, EventPropagation,
};

use crate::config::Config;
use crate::db::DbFields;
use crate::ui::{
	colors::*,
	history_view::history_view,
	primitives::{
		button::icon_button, input_field::input_field, tooltip::TooltipSignals,
	},
	window_management::{closing_window, make_field_path, opening_window},
};

pub const SECRET_PLACEHOLDER: &str = "••••••••••••••••";
const LINE_WIDTH: f64 = 250.0;

pub fn view_button_slot(
	is_secret: bool,
	tooltip_signals: TooltipSignals,
	value: RwSignal<String>,
	getter: impl Fn() -> String + 'static,
) -> impl View {
	let see_btn_visible = create_rw_signal(true);
	let hide_btn_visible = create_rw_signal(false);

	let see_icon = include_str!("./icons/see.svg");
	let hide_icon = include_str!("./icons/hide.svg");

	if is_secret {
		h_stack((
			icon_button(String::from(see_icon), see_btn_visible, move |_| {
				let data = getter();
				value.set(data);
				see_btn_visible.set(false);
				hide_btn_visible.set(true);
				tooltip_signals.hide();
			})
			.on_event(EventListener::PointerEnter, move |_event| {
				if is_secret {
					tooltip_signals.show(String::from("See contents of field"));
				}
				EventPropagation::Continue
			})
			.on_event(EventListener::PointerLeave, move |_| {
				if is_secret {
					tooltip_signals.hide();
				}
				EventPropagation::Continue
			}),
			icon_button(String::from(hide_icon), hide_btn_visible, move |_| {
				value.set(String::from(SECRET_PLACEHOLDER));
				see_btn_visible.set(true);
				hide_btn_visible.set(false);
				tooltip_signals.hide();
			})
			.on_event(EventListener::PointerEnter, move |_event| {
				if is_secret {
					tooltip_signals.show(String::from("Hide contents of field"));
				}
				EventPropagation::Continue
			})
			.on_event(EventListener::PointerLeave, move |_| {
				if is_secret {
					tooltip_signals.hide();
				}
				EventPropagation::Continue
			}),
		))
	} else {
		h_stack((label(|| ""),))
	}
}

pub fn clipboard_button_slot(
	tooltip_signals: TooltipSignals,
	getter: impl Fn() -> String + 'static,
) -> impl View {
	let clipboard_icon = include_str!("./icons/clipboard.svg");

	container(icon_button(
		String::from(clipboard_icon),
		create_rw_signal(true),
		move |_| {
			let data = getter();
			let _ = Clipboard::set_contents(data);
		},
	))
	.on_event(EventListener::PointerEnter, move |_event| {
		tooltip_signals.show(String::from("Copy to clipboard"));
		EventPropagation::Continue
	})
	.on_event(EventListener::PointerLeave, move |_| {
		tooltip_signals.hide();
		EventPropagation::Continue
	})
}

#[allow(clippy::too_many_arguments)]
fn save(
	id: usize,
	field: DbFields,
	value: RwSignal<String>,
	is_secret: bool,
	tooltip_signals: TooltipSignals,
	edit_btn_visible: RwSignal<bool>,
	save_btn_visible: RwSignal<bool>,
	input_id: Id,
	set_list: WriteSignal<im::Vector<(usize, &'static str, usize)>>,
	config: Config,
) {
	config.db.write().unwrap().edit_field(id, &field, value.get());
	if field == DbFields::Title {
		let new_list = config.db.read().unwrap().get_list();
		set_list.update(|list: &mut im::Vector<(usize, &'static str, usize)>| {
			*list = new_list;
		});
	}

	edit_btn_visible.set(true);
	save_btn_visible.set(false);
	tooltip_signals.hide();
	input_id.request_focus();

	if is_secret {
		value.set(String::from(SECRET_PLACEHOLDER));
	}
}

fn list_item(
	id: usize,
	field: DbFields,
	is_secret: bool,
	tooltip_signals: TooltipSignals,
	set_list: WriteSignal<im::Vector<(usize, &'static str, usize)>>,
	config: Config,
) -> impl View {
	let edit_btn_visible = create_rw_signal(true);
	let save_btn_visible = create_rw_signal(false);
	let history_btn_visible = create_rw_signal(true);
	let hide_history_btn_visible = create_rw_signal(false);
	let reset_text = create_rw_signal(String::from(""));

	let value = if is_secret {
		create_rw_signal(String::from(SECRET_PLACEHOLDER))
	} else {
		create_rw_signal(config.db.read().unwrap().get_last_by_field(&id, &field))
	};

	let is_dyn_field = matches!(field, DbFields::Fields(_));

	let edit_icon = include_str!("./icons/edit.svg");
	let revert_icon = include_str!("./icons/revert.svg");
	let save_icon = include_str!("./icons/save.svg");
	let history_icon = include_str!("./icons/history.svg");
	let hide_history_icon = include_str!("./icons/hide_history.svg");
	let delete_icon = include_str!("./icons/delete.svg");

	let config_edit = config.clone();
	let config_save = config.clone();
	let config_submit = config.clone();
	let config_viewbtn = config.clone();

	let field_title = match field {
		DbFields::Fields(_) => {
			config.db.read().unwrap().get_name_of_field(&id, &field)
		}
		other => format!("{}", other),
	};
	let field_name = field_title.clone();

	let input = input_field(value, move |s| {
		s.width(LINE_WIDTH)
			.padding_right(30)
			.display(Display::None)
			.apply_if(save_btn_visible.get(), |s| s.display(Display::Flex))
	});
	let input_id = input.id();

	let input_line = h_stack((
		input.on_event(EventListener::KeyDown, move |event| {
			let key = match event {
				Event::KeyDown(k) => k.key.physical_key,
				_ => PhysicalKey::Code(KeyCode::F35),
			};

			if key == PhysicalKey::Code(KeyCode::Escape) {
				value.set(reset_text.get());
				edit_btn_visible.set(true);
				save_btn_visible.set(false);
			}

			if key == PhysicalKey::Code(KeyCode::Enter) {
				save(
					id,
					field,
					value,
					is_secret,
					tooltip_signals,
					edit_btn_visible,
					save_btn_visible,
					input_id,
					set_list,
					config_submit.clone(),
				);
			}
			EventPropagation::Continue
		}),
		container(
			svg(move || String::from(revert_icon)).style(|s| s.width(16).height(16)),
		)
		.on_click(move |_| {
			value.set(reset_text.get());
			edit_btn_visible.set(true);
			save_btn_visible.set(false);
			tooltip_signals.hide();
			EventPropagation::Continue
		})
		.on_event(EventListener::PointerEnter, move |_event| {
			tooltip_signals.show(String::from("Revert field"));
			EventPropagation::Continue
		})
		.on_event(EventListener::PointerLeave, move |_| {
			tooltip_signals.hide();
			EventPropagation::Continue
		})
		.style(|s| {
			s.position(Position::Absolute)
				.z_index(5)
				.display(Display::Flex)
				.items_center()
				.justify_center()
				.inset_top(0)
				.inset_right(0)
				.inset_bottom(0)
				.width(30)
				.cursor(CursorStyle::Pointer)
		}),
	));

	let history_button_slot = if is_secret {
		let config_history = config.clone();
		h_stack((
			icon_button(String::from(history_icon), history_btn_visible, move |_| {
				let config_history_inner = config_history.clone();
				tooltip_signals.hide();
				let window_title = format!("{} Field History", field_title);

				opening_window(
					move || {
						let dates = config_history_inner
							.db
							.read()
							.unwrap()
							.get_history_dates(&id, &field);
						history_btn_visible.set(false);
						hide_history_btn_visible.set(true);

						history_view(id, field, dates, config_history_inner.clone())
					},
					make_field_path(id, &field),
					window_title,
					Size::new(350.0, 300.0),
					move || {
						history_btn_visible.set(true);
						hide_history_btn_visible.set(false);
					},
				);
			})
			.on_event(EventListener::PointerEnter, move |_event| {
				if is_secret {
					tooltip_signals.show(String::from("See history of field"));
				}
				EventPropagation::Continue
			})
			.on_event(EventListener::PointerLeave, move |_| {
				if is_secret {
					tooltip_signals.hide();
				}
				EventPropagation::Continue
			}),
			icon_button(
				String::from(hide_history_icon),
				hide_history_btn_visible,
				move |_| {
					closing_window(make_field_path(id, &field), || {
						history_btn_visible.set(true);
						hide_history_btn_visible.set(false);
					});
				},
			)
			.on_event(EventListener::PointerEnter, move |_event| {
				if is_secret {
					tooltip_signals.show(String::from("Hide history of field"));
				}
				EventPropagation::Continue
			})
			.on_event(EventListener::PointerLeave, move |_| {
				if is_secret {
					tooltip_signals.hide();
				}
				EventPropagation::Continue
			}),
		))
	} else {
		h_stack((label(|| ""),))
	};

	let delete_button_slot = if is_dyn_field {
		container(
			icon_button(
				String::from(delete_icon),
				create_rw_signal(true),
				move |_| {
					tooltip_signals.hide();
					// TODO: confirm and delete this field
				},
			)
			.on_event(EventListener::PointerEnter, move |_event| {
				tooltip_signals.show(String::from("Delete this field"));
				EventPropagation::Continue
			})
			.on_event(EventListener::PointerLeave, move |_| {
				tooltip_signals.hide();
				EventPropagation::Continue
			}),
		)
	} else {
		container(label(|| ""))
	};

	h_stack((
		container(label(move || field_name.clone()))
			.style(move |s| s.flex().width(70).justify_content(AlignContent::End))
			.on_click_stop(move |_| {
				input_id.request_focus();
			}),
		h_stack((
			input_line,
			label(move || value.get()).style(move |s| {
				s.width(LINE_WIDTH)
					.padding_top(5)
					.padding_right(6)
					.padding_left(6)
					.padding_bottom(5)
					.border_bottom(1)
					.border_color(C_TEXT_TOP)
					.display(Display::Flex)
					.apply_if(save_btn_visible.get(), |s| s.display(Display::None))
			}),
		)),
		h_stack((
			icon_button(String::from(edit_icon), edit_btn_visible, move |_| {
				reset_text.set(value.get());
				edit_btn_visible.set(false);
				save_btn_visible.set(true);
				tooltip_signals.hide();
				if is_secret {
					value
						.set(config_edit.db.read().unwrap().get_last_by_field(&id, &field));
				}
				input_id.request_focus();
			}),
			icon_button(String::from(save_icon), save_btn_visible, move |_| {
				save(
					id,
					field,
					value,
					is_secret,
					tooltip_signals,
					edit_btn_visible,
					save_btn_visible,
					input_id,
					set_list,
					config_save.clone(),
				);
			}),
		))
		.on_event(EventListener::PointerEnter, move |_event| {
			let text = if edit_btn_visible.get() {
				"Edit this field"
			} else {
				"Save to database"
			};
			tooltip_signals.show(String::from(text));
			EventPropagation::Continue
		})
		.on_event(EventListener::PointerLeave, move |_| {
			tooltip_signals.hide();
			EventPropagation::Continue
		}),
		clipboard_button_slot(tooltip_signals, move || {
			config.db.read().unwrap().get_last_by_field(&id, &field)
		}),
		view_button_slot(is_secret, tooltip_signals, value, move || {
			config_viewbtn.db.read().unwrap().get_last_by_field(&id, &field)
		}),
		history_button_slot,
		delete_button_slot,
	))
	.style(|s| s.align_items(AlignItems::Center).width_full().gap(4.0, 0.0))
}

fn new_field(tooltip_signals: TooltipSignals) -> impl View {
	let show_add_field_line = create_rw_signal(false);
	let show_add_btn = create_rw_signal(true);
	let show_minus_btn = create_rw_signal(false);

	let add_icon = include_str!("./icons/add.svg");
	let minus_icon = include_str!("./icons/minus.svg");

	v_stack((
		// TODO: create the UI for new field form
		h_stack((label(|| "Name:"), label(|| "name_field"), label(|| "add_btn")))
			.style(move |s| {
				s.display(Display::None)
					.apply_if(show_add_field_line.get(), |s| s.display(Display::Flex))
			}),
		icon_button(String::from(add_icon), show_add_btn, move |_| {
			tooltip_signals.hide();
			show_add_field_line.set(true);
			show_add_btn.set(false);
			show_minus_btn.set(true);
		})
		.on_event(EventListener::PointerEnter, move |_event| {
			tooltip_signals.show(String::from("Add a new field"));
			EventPropagation::Continue
		})
		.on_event(EventListener::PointerLeave, move |_| {
			tooltip_signals.hide();
			EventPropagation::Continue
		}),
		icon_button(String::from(minus_icon), show_minus_btn, move |_| {
			tooltip_signals.hide();
			show_add_field_line.set(false);
			show_add_btn.set(true);
			show_minus_btn.set(false);
		})
		.on_event(EventListener::PointerEnter, move |_event| {
			tooltip_signals.show(String::from("Hide the new field form"));
			EventPropagation::Continue
		})
		.on_event(EventListener::PointerLeave, move |_| {
			tooltip_signals.hide();
			EventPropagation::Continue
		}),
	))
	.style(|s| s.align_items(AlignItems::Center).width_full().gap(4.0, 0.0))
}

pub fn detail_view(
	id: usize,
	tooltip_signals: TooltipSignals,
	set_list: WriteSignal<im::Vector<(usize, &'static str, usize)>>,
	config: Config,
) -> impl View {
	let password_icon = include_str!("./icons/password.svg");

	let field_list = config.db.read().unwrap().get_fields(&id);
	let dyn_fields = field_list.iter().map(|field| {
		list_item(id, *field, true, tooltip_signals, set_list, config.clone())
	});

	v_stack((
		h_stack((
			svg(move || String::from(password_icon))
				.style(|s| s.width(24).height(24)),
			label(move || String::from("Details")).style(|s| s.font_size(24.0)),
		))
		.style(|s| {
			s.align_items(AlignItems::Center)
				.gap(5, 0)
				.margin_top(15)
				.margin_bottom(20)
		}),
		v_stack((
			list_item(
				id,
				DbFields::Title,
				false,
				tooltip_signals,
				set_list,
				config.clone(),
			),
			list_item(
				id,
				DbFields::Url,
				false,
				tooltip_signals,
				set_list,
				config.clone(),
			),
			list_item(
				id,
				DbFields::Username,
				true,
				tooltip_signals,
				set_list,
				config.clone(),
			),
			list_item(
				id,
				DbFields::Password,
				true,
				tooltip_signals,
				set_list,
				config.clone(),
			),
			// TODO: make this a virtual list so we can edit the fields
			static_list(dyn_fields),
			new_field(tooltip_signals),
		))
		.style(|s| s.gap(0, 5)),
	))
	.style(|s| {
		s.padding(8.0)
			.width_full()
			.justify_content(AlignContent::Center)
			.align_items(AlignItems::Center)
	})
}
