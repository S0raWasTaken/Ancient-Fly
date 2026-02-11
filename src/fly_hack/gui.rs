#![allow(non_snake_case)]
#![allow(static_mut_refs)]

use std::ffi::CString;
use std::{f32, ptr};

use dear_imgui_glow::GlowRenderer;
use dear_imgui_rs::{Condition, Context, StyleColor, WindowFlags};

use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{
    wglGetCurrentContext, wglGetProcAddress,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::core::{BOOL, PCSTR, w};

use minhook::MinHook;

use crate::fly_hack::fly_logic::STATE;

unsafe fn load_gl() -> glow::Context {
    let opengl32 = unsafe { GetModuleHandleW(w!("opengl32.dll")).unwrap() };

    unsafe {
        glow::Context::from_loader_function(|name| {
            let cname = CString::new(name).unwrap();

            if let Some(f) = wglGetProcAddress(PCSTR(cname.as_ptr().cast())) {
                return f as *const _;
            }

            GetProcAddress(opengl32, PCSTR(cname.as_ptr().cast()))
                .map_or(ptr::null(), |f| f as *const _)
        })
    }
}

struct Gui {
    ctx: Context,
    renderer: GlowRenderer,
}

impl Gui {
    fn init_if_needed() -> Option<&'static mut Gui> {
        static mut GUI: Option<Gui> = None;

        unsafe {
            if GUI.is_none() {
                if wglGetCurrentContext().is_invalid() {
                    return None;
                }

                let glow = load_gl();
                let mut ctx = Context::create();

                {
                    let style = ctx.style_mut();
                    style.set_color(StyleColor::WindowBg, [0.0, 0.0, 0.0, 0.0]);
                    style.set_color(StyleColor::Text, [1.0, 1.0, 1.0, 1.0]);
                    style
                        .set_color(StyleColor::Separator, [1.0, 1.0, 1.0, 0.3]);
                    style.set_color(StyleColor::Border, [0.0, 0.0, 0.0, 0.0]);

                    style.set_window_border_size(0.0);
                    style.set_window_rounding(0.2);
                }

                let renderer = GlowRenderer::new(glow, &mut ctx).ok()?;

                GUI = Some(Gui { ctx, renderer });
            }

            GUI.as_mut()
        }
    }

    fn render(&mut self) {
        let io = self.ctx.io_mut();
        io.set_display_size([800.0, 600.0]);
        io.set_delta_time(1.0 / 60.0);

        let ui = self.ctx.frame();

        ui.window("rd-132211 fly")
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_COLLAPSE
                    | WindowFlags::ALWAYS_AUTO_RESIZE,
            )
            .position([5.0, 511.0], Condition::Always)
            .build(|| {
                shadow_text(ui, "S0ra's rd-132211 Fly Hack");
                ui.separator();
                shadow_text(ui, &unsafe { format!("State: {STATE}",) });
                shadow_text(ui, "Double tap Spacebar to toggle");
                shadow_text(ui, "Press Shift to descend");
            });

        let draw_data = self.ctx.render();
        self.renderer.render(draw_data).ok();
    }
}

fn shadow_text(ui: &dear_imgui_rs::Ui, text: &str) {
    let draw = ui.get_window_draw_list();
    let pos = ui.cursor_screen_pos();

    // Shadow
    draw.add_text([pos[0] + 2.0, pos[1] + 1.0], [0.0, 0.0, 0.0, 0.8], text);

    // Foreground
    draw.add_text(pos, [1.0, 1.0, 1.0, 1.0], text);

    let font = ui.current_font();
    let font_size = ui.current_font_size();
    let size = font.calc_text_size(font_size, f32::MAX, 0.0, text);

    ui.dummy(size);
}

type SwapBuffersFn = unsafe extern "system" fn(HDC) -> BOOL;
static mut ORIGINAL_SWAP_BUFFERS: Option<SwapBuffersFn> = None;

unsafe extern "system" fn hooked_swap_buffers(hdc: HDC) -> BOOL {
    unsafe {
        if let Some(gui) = Gui::init_if_needed() {
            gui.render();
        }

        ORIGINAL_SWAP_BUFFERS.map_or(BOOL(0), |f| f(hdc))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn install_hook() -> BOOL {
    unsafe {
        let Ok(gdi32) = GetModuleHandleW(w!("gdi32.dll")) else {
            return BOOL(0);
        };

        let Some(target) =
            GetProcAddress(gdi32, PCSTR(c"SwapBuffers".as_ptr().cast::<u8>()))
        else {
            return BOOL(0);
        };

        let Ok(original) = MinHook::create_hook(
            target as *mut _,
            hooked_swap_buffers as *mut _,
        ) else {
            return BOOL(0);
        };

        ORIGINAL_SWAP_BUFFERS = Some(std::mem::transmute::<
            *mut std::ffi::c_void,
            unsafe extern "system" fn(
                windows::Win32::Graphics::Gdi::HDC,
            ) -> BOOL,
        >(original));

        if MinHook::enable_all_hooks().is_err() {
            return BOOL(0);
        }

        BOOL(1)
    }
}
