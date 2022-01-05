use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use gloo::events::EventListener;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Element, HtmlElement};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // Better error messages in debug mode.
    console_error_panic_hook::set_once();

    start_rendering_loop();

    Ok(())
}

fn start_rendering_loop() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let card = document.get_element_by_id("card").unwrap();

    let is_mouse_down = Rc::new(RefCell::new(false));
    let is_mouse_down_cloned = Rc::clone(&is_mouse_down);
    let is_mouse_down_cloned_2 = Rc::clone(&is_mouse_down);

    let _on_mouse_down = EventListener::new(&document, "pointerdown", move |_event| {
        *is_mouse_down.borrow_mut() = true;
    })
    .forget(); // Listen forever

    let _on_mouse_up = EventListener::new(&document, "pointerup", move |_event| {
        *is_mouse_down_cloned.borrow_mut() = false;
    })
    .forget();

    let mouse_x = Rc::new(RefCell::new(0.0_f64));
    let mouse_y = Rc::new(RefCell::new(0.0_f64));

    let mask: Mask = [false; AREA];
    let mask = Rc::new(RefCell::new(mask));
    let mask_cloned = Rc::clone(&mask);

    let _on_mouse_move = EventListener::new(&card, "pointermove", move |e| {
        let event = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
        let rect = event
            .target()
            .expect("mouse event doesn't have a target")
            .dyn_into::<HtmlElement>()
            .expect("event target should be of type HtmlElement")
            .get_bounding_client_rect();

        let x = (event.client_x() as f64) - rect.left();
        let y = (event.client_y() as f64) - rect.top();

        *mouse_x.borrow_mut() = x;
        *mouse_y.borrow_mut() = y;

        let mask_index: usize = calc_mask_index_1d(x, y);

        if *is_mouse_down_cloned_2.borrow() && mask_index < AREA {
            let mut borrowed_mask = *mask.borrow_mut();
            borrowed_mask[mask_index] = true;
            *mask.borrow_mut() = borrowed_mask;
        }
    })
    .forget();

    // Rendering loop inspired by
    // https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame_count.html
    let rendering_callback_cell_one = Rc::new(RefCell::new(None));
    let rendering_callback_cell_two = rendering_callback_cell_one.clone();

    *rendering_callback_cell_two.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        render(&card, mask_cloned.borrow());

        request_animation_frame(rendering_callback_cell_one.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(rendering_callback_cell_two.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    let window = web_sys::window().unwrap();

    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Unable to `requestAnimationFrame`");
}

const GLYPH_WIDTH: f64 = 10.0;
const GLYPH_HEIGHT: f64 = 15.0;
const LINE_SPACING: f64 = 13.0;

// Magic numbers, don't ask. Call it resourceful problem solving, if you will.
const WIDTH_COMPENSATION: f64 = GLYPH_WIDTH / 115.0;
const HEIGHT_COMPENSATION: f64 = (GLYPH_HEIGHT + LINE_SPACING) / 868.0;

fn calc_mask_index_1d(x: f64, y: f64) -> usize {
    let x_index = (x * WIDTH_COMPENSATION) as usize - 1;
    let y_index = (y * HEIGHT_COMPENSATION) as usize;
    gloo::console::log!(x_index, y_index, y_index * WIDTH + x_index);
    y_index * WIDTH + x_index
}

const WIDTH: usize = 40;
const HEIGHT: usize = 6;
const AREA: usize = WIDTH * HEIGHT;

type Mask = [bool; AREA];
type Characters = [char; AREA];

fn render(target: &Element, mask: Ref<Mask>) {
    let characters = get_characters(*mask);
    let string: String = characters.iter().collect();
    target.set_text_content(Some(&string));
}

const MESSAGE: [&str; HEIGHT] = [
    "",
    "    Dear people at Oxide, ",
    "    you are an inspiration! ",
    "    It'd be a dream to work with you. ",
    "    Would you like to meet? ",
    "",
];

const PADDING: usize = 2;

fn get_characters(mask: Mask) -> Characters {
    let mut characters: Characters = ['x'; AREA];

    for index in 0..AREA {
        let x_index = index % WIDTH;

        if x_index == 0 {
            characters[index] = '\n';
        } else {
            let y_index = index / WIDTH;
            let sentence = MESSAGE[y_index];
            let is_message_index = x_index > PADDING && y_index != 0 && y_index != sentence.len();
            let is_even = x_index % 2 == 0;

            let ferrous_character = match is_even {
                true => 'e',
                false => 'F',
            };
            let oxide_character = match is_even {
                true => 'x',
                false => '0',
            };

            characters[index] = if mask[index] {
                if is_message_index {
                    let message_row = sentence.as_bytes();
                    let row_has_character = x_index < message_row.len();
                    match row_has_character {
                        true => message_row[x_index] as char,
                        false => ferrous_character,
                    }
                } else {
                    ferrous_character
                }
            } else {
                oxide_character
            }
        }
    }

    characters
}
