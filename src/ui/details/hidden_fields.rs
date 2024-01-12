use floem::{
	event::EventListener,
	reactive::{create_rw_signal, ReadSignal, RwSignal, WriteSignal},
	style::{CursorStyle, Display},
	view::View,
	views::{
		container, h_stack, label, svg, v_stack, virtual_stack, Decorators,
		VirtualDirection, VirtualItemSize,
	},
	EventPropagation,
};

use crate::{
	config::Config,
	db::DbFields,
	ui::{
		colors::*,
		details::list_item::{list_item, ListItem},
		primitives::tooltip::TooltipSignals,
	},
};

pub fn hidden_fields(
	id: usize,
	hidden_field_list: ReadSignal<im::Vector<DbFields>>,
	tooltip_signals: TooltipSignals,
	set_list: WriteSignal<im::Vector<(usize, &'static str, usize)>>,
	main_scroll_to: RwSignal<f32>,
	config: Config,
) -> impl View {
	let is_expanded = create_rw_signal(false);

	let expand_icon = include_str!("../icons/expand.svg");
	let contract_icon = include_str!("../icons/contract.svg");

	let hidden_fields_len = hidden_field_list.get().len();

	if hidden_fields_len > 0 {
		v_stack((
			v_stack((
				container(label(|| "").style(|s| {
					s.border_top(1).border_color(C_BG_MAIN_BORDER).height(1).width(252)
				}))
				.style(|s| s.justify_center().margin_bottom(10)),
				virtual_stack(
					VirtualDirection::Vertical,
					VirtualItemSize::Fixed(Box::new(|| 35.0)),
					move || hidden_field_list.get(),
					move |item| *item,
					move |field| {
						list_item(ListItem {
							id,
							field,
							is_secret: true,
							is_hidden: true,
							tooltip_signals,
							set_list,
							config: config.clone(),
						})
						.style(|s| s.padding_bottom(5))
					},
				)
				.style(|s| s.display(Display::Flex)),
			))
			.style(move |s| {
				s.display(Display::None)
					.margin_bottom(10)
					.margin_top(10)
					.apply_if(is_expanded.get(), |s| s.display(Display::Flex))
			}),
			h_stack((
				label(|| "").style(|s| {
					s.border_top(1).border_color(C_BG_MAIN_BORDER).height(1).width(120)
				}),
				h_stack((
					svg(move || String::from(expand_icon))
						.style(move |s| {
							s.width(12)
								.height(12)
								.cursor(CursorStyle::Pointer)
								.display(Display::Flex)
								.apply_if(is_expanded.get(), |s| s.display(Display::None))
						})
						.on_click(move |_| {
							is_expanded.set(true);
							main_scroll_to.set(100.0);
							tooltip_signals.hide();
							EventPropagation::Continue
						})
						.on_event(EventListener::PointerEnter, move |_event| {
							tooltip_signals.show(format!(
								"Show {} hidden field{}",
								hidden_fields_len,
								if hidden_fields_len > 1 { "s" } else { "" }
							));
							EventPropagation::Continue
						})
						.on_event(EventListener::PointerLeave, move |_| {
							tooltip_signals.hide();
							EventPropagation::Continue
						}),
					svg(move || String::from(contract_icon))
						.style(move |s| {
							s.width(12)
								.height(12)
								.cursor(CursorStyle::Pointer)
								.display(Display::None)
								.apply_if(is_expanded.get(), |s| s.display(Display::Flex))
						})
						.on_click(move |_| {
							is_expanded.set(false);
							tooltip_signals.hide();
							EventPropagation::Continue
						})
						.on_event(EventListener::PointerEnter, move |_event| {
							tooltip_signals.show(String::from("Hide hidden field"));
							EventPropagation::Continue
						})
						.on_event(EventListener::PointerLeave, move |_| {
							tooltip_signals.hide();
							EventPropagation::Continue
						}),
				)),
				label(|| "").style(|s| {
					s.border_top(1).border_color(C_BG_MAIN_BORDER).height(1).width(120)
				}),
			))
			.style(|s| s.flex().items_center().justify_center().gap(4, 0)),
		))
	} else {
		h_stack((label(|| ""),))
	}
}
