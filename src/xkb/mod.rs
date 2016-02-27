
pub mod ffi;

#[cfg(feature = "x11")]
pub mod x11;
pub mod keysyms;

pub use xkb::keysyms::*;
use xkb::ffi::*;

use libc::{self, c_int, c_uint, c_char, c_void};
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use std::str;
use std::slice;
use std::mem;
use std::iter::Iterator;
use std::path::{Path};


/// A number used to represent a physical key on a keyboard.
///
/// A standard PC-compatible keyboard might have 102 keys.  An appropriate
/// keymap would assign each of them a keycode, by which the user should
/// refer to the key throughout the library.
///
/// Historically, the X11 protocol, and consequentially the XKB protocol,
/// assign only 8 bits for keycodes.  This limits the number of different
/// keys that can be used simultaneously in a single keymap to 256
/// (disregarding other limitations).  This library does not share this limit;
/// keycodes beyond 255 ('extended keycodes') are not treated specially.
/// Keymaps and applications which are compatible with X11 should not use
/// these keycodes.
///
/// The values of specific keycodes are determined by the keymap and the
/// underlying input system.  For example, with an X11-compatible keymap
/// and Linux evdev scan codes (see linux/input.h), a fixed offset is used:
///
/// let keycode_A: Keycode = KEY_A + 8;
///
/// @sa xkb_keycode_is_legal_ext() xkb_keycode_is_legal_x11()
pub type Keycode = u32;


/// A number used to represent the symbols generated from a key on a keyboard.
///
/// A key, represented by a keycode, may generate different symbols according
/// to keyboard state.  For example, on a QWERTY keyboard, pressing the key
/// labled \<A\> generates the symbol 'a'.  If the Shift key is held, it
/// generates the symbol 'A'.  If a different layout is used, say Greek,
/// it generates the symbol 'α'.  And so on.
///
/// Each such symbol is represented by a keysym.  Note that keysyms are
/// somewhat more general, in that they can also represent some "function",
/// such as "Left" or "Right" for the arrow keys.  For more information,
/// see:
/// http://www.x.org/releases/X11R7.7/doc/xproto/x11protocol.html#keysym_encoding
///
/// Specifically named keysyms can be found in the
/// xkbcommon/xkbcommon-keysyms.h header file.  Their name does not include
/// the XKB_KEY_ prefix.
///
/// Besides those, any Unicode/ISO 10646 character in the range U0100 to
/// U10FFFF can be represented by a keysym value in the range 0x01000100 to
/// 0x0110FFFF.  The name of Unicode keysyms is "U<codepoint>", e.g. "UA1B2".
///
/// The name of other unnamed keysyms is the hexadecimal representation of
/// their value, e.g. "0xabcd1234".
///
/// Keysym names are case-sensitive.
pub type Keysym = u32;



/// Index of a keyboard layout.
///
/// The layout index is a state component which detemines which _keyboard
/// layout_ active.  These may be different alphabets, different key
/// arrangements, etc.
///
/// Layout indices are consecutive.  The first layout has index 0.
///
/// Each layout is not required to have a name, and the names are not
/// guaranteed to be unique (though they are usually provided and unique).
/// Therefore, it is not safe to use the name as a unique identifier for a
/// layout.  Layout names are case-sensitive.
///
/// Layouts are also called "groups" by XKB.
///
/// @sa xkb_keymap_num_layouts() xkb_keymap_num_layouts_for_key()
pub type LayoutIndex = u32;
/// A mask of layout indices
pub type LayoutMask = u32;



/// Index of a shift level.
///
/// Any key, in any layout, can have several _shift levels_  Each
/// shift level can assign different keysyms to the key.  The shift level
/// to use is chosen according to the current keyboard state; for example,
/// if no keys are pressed, the first level may be used; if the Left Shift
/// key is pressed, the second; if Num Lock is pressed, the third; and
/// many such combinations are possible (see ModIndex).
///
/// Level indices are consecutive.  The first level has index 0.
pub type LevelIndex = u32;


/// Index of a modifier.
///
/// A @e modifier is a state component which changes the way keys are
/// interpreted.  A keymap defines a set of modifiers, such as Alt, Shift,
/// Num Lock or Meta, and specifies which keys may @e activate which
/// modifiers (in a many-to-many relationship, i.e. a key can activate
/// several modifiers, and a modifier may be activated by several keys.
/// Different keymaps do this differently).
///
/// When retrieving the keysyms for a key, the active modifier set is
/// consulted; this detemines the correct shift level to use within the
/// currently active layout (see LevelIndex).
///
/// Modifier indices are consecutive.  The first modifier has index 0.
///
/// Each modifier must have a name, and the names are unique.  Therefore, it
/// is safe to use the name as a unique identifier for a modifier.
/// Modifier names are case-sensitive.
///
/// @sa xkb_keymap_num_mods()
pub type ModIndex = u32;
/// A mask of modifier indices.
pub type ModMask = u32;


/// Index of a keyboard LED.
///
/// LEDs are logical objects which may be @e active or @e inactive.  They
/// typically correspond to the lights on the keyboard. Their state is
/// determined by the current keyboard state.
///
/// LED indices are non-consecutive.  The first LED has index 0.
///
/// Each LED must have a name, and the names are unique. Therefore,
/// it is safe to use the name as a unique identifier for a LED.  The names
/// of some common LEDs are provided in the xkbcommon/xkbcommon-names.h
/// header file.  LED names are case-sensitive.
///
/// @warning A given keymap may specify an exact index for a given LED.
/// Therefore, LED indexing is not necessarily sequential, as opposed to
/// modifiers and layouts.  This means that when iterating over the LEDs
/// in a keymap using e.g. xkb_keymap_num_leds(), some indices might be
/// invalid.  Given such an index, functions like xkb_keymap_led_get_name()
/// will return NULL, and xkb_state_led_index_is_active() will return -1.
///
/// LEDs are also called "indicators" by XKB.
///
/// @sa xkb_keymap_num_leds()
pub type LedIndex = u32;
/// A mask of LED indices.
pub type LedMask = u32;


pub const KEYCODE_INVALID:u32 = 0xffffffff;
pub const LAYOUT_INVALID :u32 = 0xffffffff;
pub const LEVEL_INVALID  :u32 = 0xffffffff;
pub const MOD_INVALID    :u32 = 0xffffffff;
pub const LED_INVALID    :u32 = 0xffffffff;

pub const KEYCODE_MAX    :u32 = 0xfffffffe;


pub type KeysymFlags = u32;
pub const KEYSYM_NO_FLAGS: u32 = 0;
pub const KEYSYM_CASE_INSENSITIVE: u32 = (1 << 0);

/// Flags for context creation.
pub type ContextFlags = u32;
/// Do not apply any context flags.
pub const CONTEXT_NO_FLAGS: u32 = 0;
/// Create this context with an empty include path.
pub const CONTEXT_NO_DEFAULT_INCLUDES: u32 = (1 << 0);
/// Don't take RMLVO names from the environment.
pub const CONTEXT_NO_ENVIRONMENT_NAMES: u32 = (1 << 1);


#[repr(C)]
pub enum LogLevel {
    Critical = 10,
    Error = 20,
    Warning = 30,
    Info = 40,
    Debug = 50,
}


/// Flags for keymap compilation.
pub type KeymapCompileFlags = u32;
/// Do not apply any flags.
pub const KEYMAP_COMPILE_NO_FLAGS: u32 = 0;


/// The possible keymap formats.
pub type KeymapFormat = u32;
/// The current/classic XKB text format, as generated by xkbcomp -xkb.
pub const KEYMAP_FORMAT_TEXT_V1: u32 = 1;
/// Get the keymap as a string in the format from which it was created.
/// @sa xkb_keymap_get_as_string()
pub const KEYMAP_FORMAT_USE_ORIGINAL: u32 = 0xffffffff;


/// Specifies the direction of the key (press / release).
#[repr(C)]
pub enum KeyDirection {
/// the key was released
    Up,
/// the key was pressed
    Down
}



/// Modifier and layout types for state objects.  This enum is bitmaskable,
/// e.g. (XKB_STATE_MODS_DEPRESSED | XKB_STATE_MODS_LATCHED) is valid to
/// exclude locked modifiers.
///
/// In XKB, the DEPRESSED components are also known as 'base'.
pub type StateComponent = u32;
/// Depressed modifiers, i.e. a key is physically holding them.
pub const STATE_MODS_DEPRESSED: u32 = (1 << 0);
/// Latched modifiers, i.e. will be unset after the next non-modifier
///  key press.
pub const STATE_MODS_LATCHED: u32 = (1 << 1);
/// Locked modifiers, i.e. will be unset after the key provoking the
///  lock has been pressed again.
pub const STATE_MODS_LOCKED: u32 = (1 << 2);
/// Effective modifiers, i.e. currently active and affect key
///  processing (derived from the other state components).
///  Use this unless you explictly care how the state came about.
pub const STATE_MODS_EFFECTIVE: u32 = (1 << 3);
/// Depressed layout, i.e. a key is physically holding it.
pub const STATE_LAYOUT_DEPRESSED: u32 = (1 << 4);
/// Latched layout, i.e. will be unset after the next non-modifier
///  key press.
pub const STATE_LAYOUT_LATCHED: u32 = (1 << 5);
/// Locked layout, i.e. will be unset after the key provoking the lock
///  has been pressed again.
pub const STATE_LAYOUT_LOCKED: u32 = (1 << 6);
/// Effective layout, i.e. currently active and affects key processing
///  (derived from the other state components).
///  Use this unless you explictly care how the state came about.
pub const STATE_LAYOUT_EFFECTIVE: u32 = (1 << 7);
/// LEDs (derived from the other state components).
pub const STATE_LEDS: u32 = (1 << 8);



/// Match flags for xkb_state_mod_indices_are_active and
/// xkb_state_mod_names_are_active, specifying how the conditions for a
/// successful match.  XKB_STATE_MATCH_NON_EXCLUSIVE is bitmaskable with
/// the other modes.
pub type StateMatch = u32;
/// Returns true if any of the modifiers are active.
pub const STATE_MATCH_ANY: u32 = (1 << 0);
/// Returns true if all of the modifiers are active.
pub const STATE_MATCH_ALL: u32 = (1 << 1);
/// Makes matching non-exclusive, i.e. will not return false if a
///  modifier not specified in the arguments is active.
pub const STATE_MATCH_NON_EXCLUSIVE: u32 = (1 << 16);

pub const MOD_NAME_SHIFT: &'static str = "Shift";
pub const MOD_NAME_CAPS: &'static str = "Lock";
pub const MOD_NAME_CTRL: &'static str = "Control";
pub const MOD_NAME_ALT: &'static str = "Mod1";
pub const MOD_NAME_NUM: &'static str = "Mod2";
pub const MOD_NAME_LOGO: &'static str = "Mod4";

pub const LED_NAME_CAPS: &'static str = "Caps Lock";
pub const LED_NAME_NUM: &'static str = "Num Lock";
pub const LED_NAME_SCROLL: &'static str = "Scroll Lock";




/// Test whether a value is a valid extended keycode.
/// @sa xkb_keycode_t
pub fn keycode_is_legal_ext(key: u32) -> bool {
    key <= KEYCODE_MAX
}


/// Names to compile a keymap with, also known as RMLVO.
///
/// The names are the common configuration values by which a user picks
/// a keymap.
///
/// If the entire struct is NULL, then each field is taken to be NULL.
/// You should prefer passing NULL instead of choosing your own defaults.
pub fn keycode_is_legal_x11(key: u32) -> bool {
    key >= 8 && key <= 255
}




/// Get the name of a keysym.
pub fn keysym_get_name(keysym: Keysym) -> String {
    unsafe {
        let mut buf: &mut [c_char] = &mut [0; 64];
        let ptr = &mut buf[0] as *mut c_char;
        let len = xkb_keysym_get_name(keysym, ptr, 64);
        let slice: &[u8] = slice::from_raw_parts(
                    mem::transmute(ptr), len as usize);
        String::from_utf8_unchecked(slice.to_owned())
    }
}



/// Get a keysym from its name.
///
/// @param name The name of a keysym. See remarks in xkb_keysym_get_name();
/// this function will accept any name returned by that function.
/// @param flags A set of flags controlling how the search is done. If
/// invalid flags are passed, this will fail with XKB_KEY_NoSymbol.
///
/// If you use the XKB_KEYSYM_CASE_INSENSITIVE flag and two keysym names
/// differ only by case, then the lower-case keysym is returned.  For
/// instance, for KEY_a and KEY_A, this function would return KEY_a for the
/// case-insensitive search.  If this functionality is needed, it is
/// recommended to first call this function without this flag; and if that
/// fails, only then to try with this flag, while possibly warning the user
/// he had misspelled the name, and might get wrong results.
///
/// @returns The keysym. If the name is invalid, returns XKB_KEY_NoSymbol.
///
/// @sa xkb_keysym_t
pub fn keysym_from_name(name: &str, flags: KeysymFlags) -> Keysym {
    unsafe {
        let cname = CString::new(name.as_bytes().to_owned()).unwrap();
        xkb_keysym_from_name(cname.as_ptr(), flags)
    }
}


/// Get the Unicode/UTF-8 representation of a keysym.
///
/// Prefer not to use this function on keysyms obtained from an
/// xkb_state.  In this case, use xkb_state_key_get_utf8() instead.
pub fn keysym_to_utf8(keysym: Keysym) -> String {
    unsafe {
        let mut buf: &mut [c_char] = &mut [0; 8];
        let ptr = &mut buf[0] as *mut c_char;
        let len = xkb_keysym_to_utf8(keysym, ptr, 8);
        let slice: &[u8] = slice::from_raw_parts(
                    mem::transmute(ptr), len as usize);
        String::from_utf8_unchecked(slice.to_owned())
    }
}


/// Get the Unicode/UTF-32 representation of a keysym.
///
/// @returns The Unicode/UTF-32 representation of keysym, which is also
/// compatible with UCS-4.  If the keysym does not have a Unicode
/// representation, returns 0.
///
/// Prefer not to use this function on keysyms obtained from an
/// xkb_state.  In this case, use xkb_state_key_get_utf32() instead.
///
/// @sa xkb_state_key_get_utf32()
pub fn keysym_to_utf32(keysym: Keysym) -> u32 {
    unsafe { xkb_keysym_to_utf32(keysym) }
}



/// Top level library context object.
///
/// The context contains various general library data and state, like
/// logging level and include paths.
///
/// Objects are created in a specific context, and multiple contexts may
/// coexist simultaneously.  Objects from different contexts are completely
/// separated and do not share any memory or state.
pub struct Context {
    ptr: *mut xkb_context
}


impl Context {

    /// contruct a context from a raw ffi pointer. This context must already been
    /// referenced as xkb_context_unref will be called at drop time
    pub unsafe fn from_raw_ptr(ptr: *mut xkb_context) -> Context {
        Context {
            ptr: ptr
        }
    }

    /// get the raw pointer from this context
    pub fn get_raw_ptr(&self) -> *mut xkb_context {
        self.ptr
    }


    /// Create a new context.
    ///
    /// @param flags Optional flags for the context, or 0.
    ///
    /// The user may set some environment variables to affect default values in
    /// the context. See e.g. xkb_context_set_log_level() and
    /// xkb_context_set_log_verbosity().
    pub fn new(flags: ContextFlags) -> Context {
        unsafe {
            Context {
                ptr: xkb_context_new(flags)
            }
        }
    }

    /// append a new entry to the context's include path
    /// returns true on success, or false if the include path could not be added
    /// or is inaccessible
    pub fn include_path_append(&mut self, path: &Path) -> bool {
        if let Some(s) = path.to_str() {
            unsafe {
                let cstr = CString::from_vec_unchecked(
                    s.as_bytes().to_owned()
                );
                if xkb_context_include_path_append(
                        self.ptr, cstr.as_ptr()) == 1 {
                    true
                }
                else {
                    false
                }
            }
        }
        else {
            false
        }
    }


    /// Append the default include paths to the context's include path.
    ///
    /// @returns 1 on success, or 0 if the primary include path could not be added.
    ///
    /// @memberof xkb_context
    pub fn include_path_append_default(&mut self) -> bool {
        unsafe {
            if xkb_context_include_path_append_default(self.ptr) == 1 {
                true
            }
            else {
                false
            }
        }
    }


    /// Reset the context's include path to the default.
    ///
    /// Removes all entries from the context's include path, and inserts the
    /// default paths.
    ///
    /// @returns 1 on success, or 0 if the primary include path could not be added.
    ///
    /// @memberof xkb_context
    pub fn include_path_reset_defaults(&mut self) -> bool {
        unsafe {
            if xkb_context_include_path_reset_defaults(self.ptr) == 1 {
                true
            }
            else {
                false
            }
        }
    }


    /// Remove all entries from the context's include path.
    ///
    /// @memberof xkb_context
    pub fn include_path_clear(&mut self) {
        unsafe {
            xkb_context_include_path_clear(self.ptr);
        }
    }

    /// get an iterator on the include paths of this context
    pub fn include_paths<'a>(&'a self) -> ContextIncludePaths<'a> {
        unsafe {
            ContextIncludePaths {
                context: &self,
                ind: 0,
                len: xkb_context_num_include_paths(self.ptr)
            }
        }
    }


    /// Set the current logging level.
    ///
    /// @param context The context in which to set the logging level.
    /// @param level   The logging level to use.  Only messages from this level
    /// and below will be logged.
    ///
    /// The default level is XKB_LOG_LEVEL_ERROR.  The environment variable
    /// XKB_LOG_LEVEL, if set in the time the context was created, overrides the
    /// default value.  It may be specified as a level number or name.
    ///
    /// @memberof xkb_context
    pub fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            xkb_context_set_log_level(self.ptr,
                    mem::transmute(level));
        }
    }

    pub fn get_log_level(&self) -> LogLevel {
        unsafe {
            mem::transmute(xkb_context_get_log_level(self.ptr))
        }
    }


    /// Sets the current logging verbosity.
    ///
    /// The library can generate a number of warnings which are not helpful to
    /// ordinary users of the library.  The verbosity may be increased if more
    /// information is desired (e.g. when developing a new keymap).
    ///
    /// The default verbosity is 0.  The environment variable XKB_LOG_VERBOSITY,
    /// if set in the time the context was created, overrides the default value.
    ///
    /// @param context   The context in which to use the set verbosity.
    /// @param verbosity The verbosity to use.  Currently used values are
    /// 1 to 10, higher values being more verbose.  0 would result in no verbose
    /// messages being logged.
    ///
    /// Most verbose messages are of level XKB_LOG_LEVEL_WARNING or lower.
    ///
    /// @memberof xkb_context
    pub fn set_log_verbosity(&mut self, verbosity: i32) {
        unsafe {
            xkb_context_set_log_verbosity(self.ptr,
                    verbosity as c_int);
        }
    }

    pub fn get_log_verbosity(&self) -> i32 {
        unsafe {
            xkb_context_get_log_verbosity(self.ptr) as i32
        }
    }

}

impl Clone for Context {
    fn clone(&self) -> Context {
        unsafe {
            Context {
                ptr: xkb_context_ref(self.ptr)
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            xkb_context_unref(self.ptr);
        }
    }
}

pub struct ContextIncludePaths<'a> {
    context: &'a Context,
    ind: c_uint,
    len: c_uint,
}

impl<'a> Iterator for ContextIncludePaths<'a> {
    type Item = &'a Path;
    fn next(&mut self) -> Option<&'a Path> {
        if self.ind == self.len {
            None
        }
        else { unsafe {
            let ptr = xkb_context_include_path_get(self.context.ptr, self.ind);
            self.ind += 1;
            let cstr = CStr::from_ptr(ptr);
            Some(Path::new(str::from_utf8_unchecked(cstr.to_bytes())))
        }}
    }
}

#[test]
fn check_include_paths() {
    let mut c = Context::new(CONTEXT_NO_DEFAULT_INCLUDES);
    let test_path = Path::new("/");
    assert_eq!(true, c.include_path_append(&test_path));
    assert_eq!(test_path, c.include_paths().nth(0).unwrap());
}



/// Compiled keymap object.
///
/// The keymap object holds all of the static keyboard information obtained
/// from compiling XKB files.
///
/// A keymap is immutable after it is created (besides reference counts, etc.);
/// if you need to change it, you must create a new one.
pub struct Keymap {
    ptr: *mut xkb_keymap
}

impl Keymap {

    pub unsafe fn from_raw_ptr(ptr: *mut xkb_keymap) -> Keymap {
        Keymap {
            ptr: ptr
        }
    }

    pub fn get_raw_ptr(&self) -> *mut xkb_keymap {
        self.ptr
    }


    /// Create a keymap from RMLVO names.
    ///
    /// The primary keymap entry point: creates a new XKB keymap from a set of
    /// RMLVO (Rules + Model + Layouts + Variants + Options) names.
    ///
    /// @param context The context in which to create the keymap.
    /// @param names   The RMLVO names to use.  See xkb_rule_names.
    /// @param flags   Optional flags for the keymap, or 0.
    ///
    /// @returns A keymap compiled according to the RMLVO names, or NULL if
    /// the compilation failed.
    ///
    /// @sa xkb_rule_names
    /// @memberof xkb_keymap
    pub fn new_from_names<S: AsRef<str>+?Sized>(context: &Context,
                                                rules: &S,
                                                model: &S,
                                                layout: &S,
                                                variant: &S,
                                                options: &S,
                                                flags: KeymapCompileFlags)
            -> Keymap {
        let crules = CString::new(rules.as_ref().as_bytes()).unwrap();
        let cmodel = CString::new(model.as_ref().as_bytes()).unwrap();
        let clayout = CString::new(layout.as_ref().as_bytes()).unwrap();
        let cvariant = CString::new(variant.as_ref().as_bytes()).unwrap();
        let coptions = CString::new(options.as_ref().as_bytes()).unwrap();
        let rule_names = xkb_rule_names {
            rules: crules.as_ptr(),
            model: cmodel.as_ptr(),
            layout: clayout.as_ptr(),
            variant: cvariant.as_ptr(),
            options: coptions.as_ptr(),
        };
        unsafe {
            Keymap {
                ptr: xkb_keymap_new_from_names(context.ptr,
                        &rule_names, flags)
            }
        }
    }


    /// Create a keymap from a keymap string.
    ///
    /// This is just like xkb_keymap_new_from_file(), but instead of a file, gets
    /// the keymap as one enormous string.
    ///
    /// @see xkb_keymap_new_from_string()
    /// @memberof xkb_keymap
    pub fn new_from_string<S: AsRef<str>+?Sized>(context: &Context,
                                                string: &S,
                                                rules: &S,
                                                model: &S,
                                                layout: &S,
                                                variant: &S,
                                                options: &S,
                                                flags: KeymapCompileFlags)
            -> Keymap {
        let cstring = CString::new(string.as_ref().as_bytes()).unwrap();
        let crules = CString::new(rules.as_ref().as_bytes()).unwrap();
        let cmodel = CString::new(model.as_ref().as_bytes()).unwrap();
        let clayout = CString::new(layout.as_ref().as_bytes()).unwrap();
        let cvariant = CString::new(variant.as_ref().as_bytes()).unwrap();
        let coptions = CString::new(options.as_ref().as_bytes()).unwrap();
        let rule_names = xkb_rule_names {
            rules: crules.as_ptr(),
            model: cmodel.as_ptr(),
            layout: clayout.as_ptr(),
            variant: cvariant.as_ptr(),
            options: coptions.as_ptr(),
        };
        unsafe {
            Keymap {
                ptr: xkb_keymap_new_from_string(context.ptr,
                        cstring.as_ptr(),
                        &rule_names, flags)
            }
        }
    }


    /// Get the compiled keymap as a string.
    ///
    /// @param keymap The keymap to get as a string.
    /// @param format The keymap format to use for the string.  You can pass
    /// in the special value XKB_KEYMAP_USE_ORIGINAL_FORMAT to use the format
    /// from which the keymap was originally created.
    ///
    /// @returns The keymap as a NUL-terminated string, or NULL if unsuccessful.
    ///
    /// The returned string may be fed back into xkb_map_new_from_string() to get
    /// the exact same keymap (possibly in another process, etc.).
    ///
    /// The returned string is dynamically allocated and should be freed by the
    /// caller.
    ///
    /// @memberof xkb_keymap
    pub fn get_as_string(&self, format: KeymapFormat) -> String {
        unsafe {
            let ffistr = xkb_keymap_get_as_string(self.ptr, format);
            let cstr = CStr::from_ptr(ffistr);
            let res = String::from_utf8_unchecked(cstr.to_bytes().to_owned());
            libc::free(ffistr as *mut c_void);
            res
        }
    }


    /// Get the minimum keycode in the keymap.
    ///
    /// @sa xkb_keycode_t
    /// @memberof xkb_keymap
    pub fn min_keycode(&self) -> Keycode {
        unsafe {
            xkb_keymap_min_keycode(self.ptr)
        }
    }


    /// Get the maximum keycode in the keymap.
    ///
    /// @sa xkb_keycode_t
    /// @memberof xkb_keymap
    pub fn max_keycode(&self) -> Keycode {
        unsafe {
            xkb_keymap_max_keycode(self.ptr)
        }
    }

    pub fn mods<'a>(&'a self) -> KeymapMods<'a> {
        unsafe {
            KeymapMods {
                keymap: &self, ind: 0,
                len: xkb_keymap_num_mods(self.ptr)
            }
        }
    }


    /// Get the number of modifiers in the keymap.
    ///
    /// @sa xkb_mod_index_t
    /// @memberof xkb_keymap
    pub fn num_mods(&self) -> ModIndex {
        unsafe {
            xkb_keymap_num_mods(self.ptr)
        }
    }


    /// Get the name of a modifier by index.
    ///
    /// @returns The name.  If the index is invalid, returns NULL.
    ///
    /// @sa xkb_mod_index_t
    /// @memberof xkb_keymap
    pub fn mod_get_name<'a>(&'a self, idx: ModIndex) -> &'a str {
        unsafe {
            let ptr = xkb_keymap_mod_get_name(self.ptr, idx);
            let cstr = CStr::from_ptr(ptr);
            str::from_utf8_unchecked(cstr.to_bytes())
        }
    }


    /// Get the index of a modifier by name.
    ///
    /// @returns The index.  If no modifier with this name exists, returns
    /// XKB_MOD_INVALID.
    ///
    /// @sa xkb_mod_index_t
    /// @memberof xkb_keymap
    pub fn mod_get_index<S: AsRef<str>+?Sized>(&self, name: &S) -> ModIndex {
        unsafe {
            let cstr = CString::new(name.as_ref().as_bytes()).unwrap();
            xkb_keymap_mod_get_index(self.ptr, cstr.as_ptr())
        }
    }


    pub fn layouts<'a>(&'a self) -> KeymapLayouts<'a> {
        unsafe {
            KeymapLayouts {
                keymap: &self, ind: 0,
                len: xkb_keymap_num_layouts(self.ptr)
            }
        }
    }


    /// Get the number of layouts in the keymap.
    ///
    /// @sa xkb_layout_index_t xkb_rule_names xkb_keymap_num_layouts_for_key()
    /// @memberof xkb_keymap
    pub fn num_layouts(&self) -> LayoutIndex {
        unsafe {
            xkb_keymap_num_layouts(self.ptr)
        }
    }


    /// Get the name of a layout by index.
    ///
    /// @returns The name.  If the index is invalid, or the layout does not have
    /// a name, returns NULL.
    ///
    /// @sa xkb_layout_index_t
    /// @memberof xkb_keymap
    pub fn layout_get_name<'a>(&'a self, idx: LayoutIndex) -> &'a str {
        unsafe {
            let ptr = xkb_keymap_layout_get_name(self.ptr, idx);
            let cstr = CStr::from_ptr(ptr);
            str::from_utf8_unchecked(cstr.to_bytes())
        }
    }


    /// Get the index of a layout by name.
    ///
    /// @returns The index.  If no layout exists with this name, returns
    /// XKB_LAYOUT_INVALID.  If more than one layout in the keymap has this name,
    /// returns the lowest index among them.
    ///
    /// @memberof xkb_keymap
    pub fn layout_get_index<S: AsRef<str>+?Sized>(&self, name: &S) -> LayoutIndex {
        unsafe {
            let cstr = CString::new(name.as_ref().as_bytes()).unwrap();
            xkb_keymap_layout_get_index(self.ptr, cstr.as_ptr())
        }
    }


    pub fn leds<'a>(&'a self) -> KeymapLeds<'a> {
        unsafe {
            KeymapLeds {
                keymap: &self, ind: 0,
                len: xkb_keymap_num_leds(self.ptr)
            }
        }
    }


    /// Get the number of LEDs in the keymap.
    ///
    /// @warning The range [ 0...xkb_keymap_num_leds() ) includes all of the LEDs
    /// in the keymap, but may also contain inactive LEDs.  When iterating over
    /// this range, you need the handle this case when calling functions such as
    /// xkb_keymap_led_get_name() or xkb_state_led_index_is_active().
    ///
    /// @sa xkb_led_index_t
    /// @memberof xkb_keymap
    pub fn num_leds(&self) -> LedIndex {
        unsafe {
            xkb_keymap_num_leds(self.ptr)
        }
    }


    /// Get the name of a LED by index.
    ///
    /// @returns The name.  If the index is invalid, returns NULL.
    ///
    /// @memberof xkb_keymap
    pub fn led_get_name<'a>(&'a self, idx: LedIndex) -> &'a str {
        unsafe {
            let ptr = xkb_keymap_led_get_name(self.ptr, idx);
            let cstr = CStr::from_ptr(ptr);
            str::from_utf8_unchecked(cstr.to_bytes())
        }
    }


    /// Get the index of a LED by name.
    ///
    /// @returns The index.  If no LED with this name exists, returns
    /// XKB_LED_INVALID.
    ///
    /// @memberof xkb_keymap
    pub fn led_get_index<S: AsRef<str>+?Sized>(&self, name: &S) -> LedIndex {
        unsafe {
            let cstr = CString::new(name.as_ref().as_bytes()).unwrap();
            xkb_keymap_led_get_index(self.ptr, cstr.as_ptr())
        }
    }


    /// Get the number of layouts for a specific key.
    ///
    /// This number can be different from xkb_keymap_num_layouts(), but is always
    /// smaller.  It is the appropriate value to use when iterating over the
    /// layouts of a key.
    ///
    /// @sa xkb_layout_index_t
    /// @memberof xkb_keymap
    pub fn num_layouts_for_key(&self, key: Keycode) -> LayoutIndex {
        unsafe {
            xkb_keymap_num_layouts_for_key(self.ptr, key)
        }
    }


    /// Get the number of shift levels for a specific key and layout.
    ///
    /// If @c layout is out of range for this key (that is, larger or equal to
    /// the value returned by xkb_keymap_num_layouts_for_key()), it is brought
    /// back into range in a manner consistent with xkb_state_key_get_layout().
    ///
    /// @sa xkb_level_index_t
    /// @memberof xkb_keymap
    pub fn num_levels_for_key(&self, key: Keycode) -> LevelIndex {
        unsafe {
            xkb_keymap_num_levels_for_key(self.ptr, key)
        }
    }


    /// Get the keysyms obtained from pressing a key in a given layout and
    /// shift level.
    ///
    /// This function is like xkb_state_key_get_syms(), only the layout and
    /// shift level are not derived from the keyboard state but are instead
    /// specified explicitly.
    ///
    /// @param[in] keymap    The keymap.
    /// @param[in] key       The keycode of the key.
    /// @param[in] layout    The layout for which to get the keysyms.
    /// @param[in] level     The shift level in the layout for which to get the
    /// keysyms. This must be smaller than:
    /// @code xkb_keymap_num_layouts_for_key(keymap, key) @endcode
    /// @param[out] syms_out An immutible array of keysyms corresponding to the
    /// key in the given layout and shift level.
    ///
    /// If @c layout is out of range for this key (that is, larger or equal to
    /// the value returned by xkb_keymap_num_layouts_for_key()), it is brought
    /// back into range in a manner consistent with xkb_state_key_get_layout().
    ///
    /// @returns The number of keysyms in the syms_out array.  If no keysyms
    /// are produced by the key in the given layout and shift level, returns 0
    /// and sets syms_out to NULL.
    ///
    /// @sa xkb_state_key_get_syms()
    /// @memberof xkb_keymap
    pub fn key_get_syms_by_level<'a>(&'a self, key: Keycode,
                                     layout: LayoutIndex,
                                     level: LevelIndex)
            -> &'a [Keysym] {
        unsafe {
            let mut syms_out: *const Keysym = null_mut();
            let len = xkb_keymap_key_get_syms_by_level(self.ptr,
                                key, layout, level,
                                &mut syms_out);
            slice::from_raw_parts(syms_out, len as usize)
        }
    }


    /// Determine whether a key should repeat or not.
    ///
    /// A keymap may specify different repeat behaviors for different keys.
    /// Most keys should generally exhibit repeat behavior; for example, holding
    /// the 'a' key down in a text editor should normally insert a single 'a'
    /// character every few milliseconds, until the key is released.  However,
    /// there are keys which should not or do not need to be repeated.  For
    /// example, repeating modifier keys such as Left/Right Shift or Caps Lock
    /// is not generally useful or desired.
    ///
    /// @returns 1 if the key should repeat, 0 otherwise.
    ///
    /// @memberof xkb_keymap
    pub fn key_repeats(&self, key: Keycode) -> bool {
        unsafe {
            xkb_keymap_key_repeats(self.ptr, key) != 0
        }
    }
}

impl Clone for Keymap {
    fn clone(&self) -> Keymap {
        unsafe {
            Keymap {
                ptr: xkb_keymap_ref(self.ptr)
            }
        }
    }
}

impl Drop for Keymap {
    fn drop(&mut self) {
        unsafe {
            xkb_keymap_unref(self.ptr);
        }
    }
}

pub struct KeymapMods<'a> {
    keymap: &'a Keymap,
    ind: ModIndex,
    len: ModIndex,
}

impl<'a> Iterator for KeymapMods<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.ind == self.len {
            None
        }
        else { unsafe {
            let ptr = xkb_keymap_mod_get_name(self.keymap.ptr, self.ind);
            self.ind += 1;
            let cstr = CStr::from_ptr(ptr);
            Some(str::from_utf8_unchecked(cstr.to_bytes()))
        }}
    }
}


pub struct KeymapLayouts<'a> {
    keymap: &'a Keymap,
    ind: LayoutIndex,
    len: LayoutIndex,
}

impl<'a> Iterator for KeymapLayouts<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.ind == self.len {
            None
        }
        else { unsafe {
            let ptr = xkb_keymap_layout_get_name(self.keymap.ptr, self.ind);
            self.ind += 1;
            let cstr = CStr::from_ptr(ptr);
            Some(str::from_utf8_unchecked(cstr.to_bytes()))
        }}
    }
}


pub struct KeymapLeds<'a> {
    keymap: &'a Keymap,
    ind: LedIndex,
    len: LedIndex,
}

impl<'a> Iterator for KeymapLeds<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.ind == self.len {
            None
        }
        else { unsafe {
            let ptr = xkb_keymap_led_get_name(self.keymap.ptr, self.ind);
            self.ind += 1;
            let cstr = CStr::from_ptr(ptr);
            Some(str::from_utf8_unchecked(cstr.to_bytes()))
        }}
    }
}


/// Keyboard state object.
///
/// State objects contain the active state of a keyboard (or keyboards), such
/// as the currently effective layout and the active modifiers.  It acts as a
/// simple state machine, wherein key presses and releases are the input, and
/// key symbols (keysyms) are the output.
pub struct State {
    ptr: *mut xkb_state
}

impl State {

    pub unsafe fn from_raw_ptr(ptr: *mut xkb_state) -> State {
        State {
            ptr: ptr
        }
    }

    pub fn get_raw_ptr(&self) -> *mut xkb_state {
        self.ptr
    }


    /// Create a new keyboard state object.
    ///
    /// @param keymap The keymap which the state will use.
    ///
    /// @returns A new keyboard state object, or NULL on failure.
    ///
    /// @memberof xkb_state
    pub fn new(keymap: &Keymap) -> State {
        unsafe {
            State {
                ptr: xkb_state_new(keymap.ptr)
            }
        }
    }


    /// Get the keymap which a keyboard state object is using.
    ///
    /// @returns The keymap which was passed to xkb_state_new() when creating
    /// this state object.
    ///
    /// This function does not take a new reference on the keymap; you must
    /// explicitly reference it yourself if you plan to use it beyond the
    /// lifetime of the state.
    ///
    /// @memberof xkb_state
    pub fn get_keymap(&self) -> Keymap {
        unsafe {
            let keymap = xkb_state_get_keymap(self.ptr);
            xkb_keymap_ref(keymap);
            Keymap::from_raw_ptr(keymap)
        }
    }


    /// Update the keyboard state to reflect a given key being pressed or
    /// released.
    ///
    /// This entry point is intended for programs which track the keyboard state
    /// explictly (like an evdev client).  If the state is serialized to you by
    /// a master process (like a Wayland compositor) using functions like
    /// xkb_state_serialize_mods(), you should use xkb_state_update_mask() instead.
    /// The two functins should not generally be used together.
    ///
    /// A series of calls to this function should be consistent; that is, a call
    /// with XKB_KEY_DOWN for a key should be matched by an XKB_KEY_UP; if a key
    /// is pressed twice, it should be released twice; etc. Otherwise (e.g. due
    /// to missed input events), situations like "stuck modifiers" may occur.
    ///
    /// This function is often used in conjunction with the function
    /// xkb_state_key_get_syms() (or xkb_state_key_get_one_sym()), for example,
    /// when handling a key event.  In this case, you should prefer to get the
    /// keysyms *before* updating the key, such that the keysyms reported for
    /// the key event are not affected by the event itself.  This is the
    /// conventional behavior.
    ///
    /// @returns A mask of state components that have changed as a result of
    /// the update.  If nothing in the state has changed, returns 0.
    ///
    /// @memberof xkb_state
    ///
    /// @sa xkb_state_update_mask()
    pub fn update_key(&mut self,
                      key: Keycode,
                      direction: KeyDirection)
            -> StateComponent {
        unsafe {
            xkb_state_update_key(self.ptr, key,
                                 mem::transmute(direction))
        }
    }


    /// Update a keyboard state from a set of explicit masks.
    ///
    /// This entry point is intended for window systems and the like, where a
    /// master process holds an xkb_state, then serializes it over a wire
    /// protocol, and clients then use the serialization to feed in to their own
    /// xkb_state.
    ///
    /// All parameters must always be passed, or the resulting state may be
    /// incoherent.
    ///
    /// The serialization is lossy and will not survive round trips; it must only
    /// be used to feed slave state objects, and must not be used to update the
    /// master state.
    ///
    /// If you do not fit the description above, you should use
    /// xkb_state_update_key() instead.  The two functions should not generally be
    /// used together.
    ///
    /// @returns A mask of state components that have changed as a result of
    /// the update.  If nothing in the state has changed, returns 0.
    ///
    /// @memberof xkb_state
    ///
    /// @sa xkb_state_component
    /// @sa xkb_state_update_key
    pub fn update_mask(&mut self,
                       depressed_mods: ModMask,
                       latched_mods: ModMask,
                       locked_mods: ModMask,
                       depressed_layout: LayoutIndex,
                       latched_layout: LayoutIndex,
                       locked_layout: LayoutIndex) -> StateComponent {
        unsafe {
            xkb_state_update_mask(self.ptr,
                                  depressed_mods,
                                  latched_mods,
                                  locked_mods,
                                  depressed_layout,
                                  latched_layout,
                                  locked_layout)
        }
    }


    /// Get the keysyms obtained from pressing a particular key in a given
    /// keyboard state.
    ///
    /// Get the keysyms for a key according to the current active layout,
    /// modifiers and shift level for the key, as determined by a keyboard
    /// state.
    ///
    /// @param[in]  state    The keyboard state object.
    /// @param[in]  key      The keycode of the key.
    /// @param[out] syms_out An immutable array of keysyms corresponding the
    /// key in the given keyboard state.
    ///
    /// As an extension to XKB, this function can return more than one keysym.
    /// If you do not want to handle this case, you should use
    /// xkb_state_key_get_one_sym(), which additionally performs transformations
    /// which are specific to the one-keysym case.
    ///
    /// @returns The number of keysyms in the syms_out array.  If no keysyms
    /// are produced by the key in the given keyboard state, returns 0 and sets
    /// syms_out to NULL.
    ///
    /// @memberof xkb_state
    pub fn key_get_syms<'a>(&'a self, key: Keycode)
            -> &'a [Keysym] {
        unsafe {
            let mut syms_out: *const Keysym = null_mut();
            let len = xkb_state_key_get_syms(self.ptr,
                                key, &mut syms_out);
            slice::from_raw_parts(syms_out, len as usize)
        }
    }


    /// Get the Unicode/UTF-8 string obtained from pressing a particular key
    /// in a given keyboard state.
    ///
    /// @param[in]  state  The keyboard state object.
    /// @param[in]  key    The keycode of the key.
    /// @param[out] buffer A buffer to write the string into.
    /// @param[in]  size   Size of the buffer.
    ///
    /// @warning If the buffer passed is too small, the string is truncated
    /// (though still NUL-terminated).
    ///
    /// @returns The number of bytes required for the string, excluding the
    /// NUL byte.  If there is nothing to write, returns 0.
    ///
    /// You may check if truncation has occurred by comparing the return value
    /// with the size of @p buffer, similarly to the snprintf(3) function.
    /// You may safely pass NULL and 0 to @p buffer and @p size to find the
    /// required size (without the NUL-byte).
    ///
    /// @memberof xkb_state
    pub fn key_get_utf8(&self, key: Keycode) -> String {
        unsafe {
            let mut buf: &mut [c_char] = &mut [0; 32];
            let ptr = &mut buf[0] as *mut c_char;
            let len = xkb_state_key_get_utf8(self.ptr, key,
                        ptr, 32);
            let slice: &[u8] = slice::from_raw_parts(
                        mem::transmute(ptr), len as usize);
            String::from_utf8_unchecked(slice.to_owned())
        }
    }


    /// Get the Unicode/UTF-32 codepoint obtained from pressing a particular
    /// key in a a given keyboard state.
    ///
    /// @returns The UTF-32 representation for the key, if it consists of only
    /// a single codepoint.  Otherwise, returns 0.
    ///
    /// @memberof xkb_state
    pub fn key_get_utf32(&self, key: Keycode) -> u32 {
        unsafe {
            xkb_state_key_get_utf32(self.ptr, key)
        }
    }


    /// Get the single keysym obtained from pressing a particular key in a
    /// given keyboard state.
    ///
    /// This function is similar to xkb_state_key_get_syms(), but intended
    /// for users which cannot or do not want to handle the case where
    /// multiple keysyms are returned (in which case this function is
    /// preferred).
    ///
    /// @returns The keysym.  If the key does not have exactly one keysym,
    /// returns XKB_KEY_NoSymbol
    ///
    /// @sa xkb_state_key_get_syms()
    /// @memberof xkb_state
    pub fn key_get_one_sym(&self, key: Keycode) -> Keysym {
        unsafe {
            xkb_state_key_get_one_sym(self.ptr, key)
        }
    }


    /// Get the effective layout index for a key in a given keyboard state.
    ///
    /// @returns The layout index for the key in the given keyboard state.  If
    /// the given keycode is invalid, or if the key is not included in any
    /// layout at all, returns XKB_LAYOUT_INVALID.
    ///
    /// @invariant If the returned layout is valid, the following always holds:
    /// @code
    /// xkb_state_key_get_layout(state, key) < xkb_keymap_num_layouts_for_key(keymap, key)
    /// @endcode
    ///
    /// @memberof xkb_state
    pub fn key_get_layout(&self, key: Keycode) -> LayoutIndex {
        unsafe {
            xkb_state_key_get_layout(self.ptr, key)
        }
    }


    /// Get the effective shift level for a key in a given keyboard state and
    /// layout.
    ///
    /// @param state The keyboard state.
    /// @param key The keycode of the key.
    /// @param layout The layout for which to get the shift level.  This must be
    /// smaller than:
    /// @code xkb_keymap_num_layouts_for_key(keymap, key) @endcode
    /// usually it would be:
    /// @code xkb_state_key_get_layout(state, key) @endcode
    ///
    /// @return The shift level index.  If the key or layout are invalid,
    /// returns XKB_LEVEL_INVALID.
    ///
    /// @invariant If the returned level is valid, the following always holds:
    /// @code
    /// xkb_state_key_get_level(state, key, layout) < xkb_keymap_num_levels_for_key(keymap, key, layout)
    /// @endcode
    ///
    /// @memberof xkb_state
    pub fn key_get_level(&self, key: Keycode, layout: LayoutIndex)
            -> LevelIndex {
        unsafe {
            xkb_state_key_get_level(self.ptr, key, layout)
        }
    }

    pub fn serialize_mods(&self, components: StateComponent) -> ModMask {
        unsafe {
            xkb_state_serialize_mods(self.ptr, components)
        }
    }


    /// The counterpart to xkb_state_update_mask for modifiers, to be used on
    /// the server side of serialization.
    ///
    /// @param state      The keyboard state.
    /// @param components A mask of the modifier state components to serialize.
    /// State components other than XKB_STATE_MODS_* are ignored.
    /// If XKB_STATE_MODS_EFFECTIVE is included, all other state components are
    /// ignored.
    ///
    /// @returns A xkb_mod_mask_t representing the given components of the
    /// modifier state.
    ///
    /// This function should not be used in regular clients; please use the
    /// xkb_state_mod_*_is_active API instead.
    ///
    /// @memberof xkb_state
    pub fn serialize_layout(&self, components: StateComponent) -> LayoutIndex {
        unsafe {
            xkb_state_serialize_layout(self.ptr, components)
        }
    }


    /// Test whether a modifier is active in a given keyboard state by name.
    ///
    /// @returns 1 if the modifier is active, 0 if it is not.  If the modifier
    /// name does not exist in the keymap, returns -1.
    ///
    /// @memberof xkb_state
    pub fn mod_name_is_active<S: AsRef<str>+?Sized>(&self,
                                                    name: &S,
                                                    type_: StateComponent)
            -> bool {
        unsafe {
            let cname = CString::new(name.as_ref().as_bytes()).unwrap();
            xkb_state_mod_name_is_active(self.ptr, cname.as_ptr(), type_) != 0
        }
    }


    /// Test whether a modifier is active in a given keyboard state by index.
    ///
    /// @returns 1 if the modifier is active, 0 if it is not.  If the modifier
    /// index is invalid in the keymap, returns -1.
    ///
    /// @memberof xkb_state
    pub fn mod_index_is_active(&self, idx: ModIndex, type_: StateComponent)
            -> bool {
        unsafe {
            xkb_state_mod_index_is_active(self.ptr, idx, type_) != 0
        }
    }


    /// Test whether a modifier is consumed by keyboard state translation for
    /// a key.
    ///
    /// Some functions, like xkb_state_key_get_syms(), look at the state of
    /// the modifiers in the keymap and derive from it the correct shift level
    /// to use for the key.  For example, in a US layout, pressing the key
    /// labeled \<A\> while the Shift modifier is active, generates the keysym 'A'.
    /// In this case, the Shift modifier is said to be consumed.  However, the
    /// Num Lock modifier does not affect this translation at all, even if it
    /// active, so it is not consumed by this translation.
    ///
    /// It may be desirable for some application to not reuse consumed modifiers
    /// for further processing, e.g. for hotkeys or keyboard shortcuts. To
    /// understand why, consider some requirements from a standard shortcut
    /// mechanism, and how they are implemented:
    ///
    /// 1. The shortcut's modifiers must match exactly to the state. For example,
    ///    it is possible to bind separate actions to \<Alt\>\<Tab\> and to
    ///    \<Alt\>\<Shift\>\<Tab\>. Further, if only \<Alt\>\<Tab\> is bound to
    ///    an action, pressing \<Alt\>\<Shift\>\<Tab\> should not trigger the
    ///    shortcut.
    ///    Effectively, this means that the modifiers are compared using the
    ///    equality operator (==).
    /// 2. Only relevant modifiers are considered for the matching. For example,
    ///    Caps Lock and Num Lock should not generally affect the matching, e.g.
    ///    when matching \<Alt\>\<Tab\> against the state, it does not matter
    ///    whether Num Lock is active or not. These relevant, or significant,
    ///    modifiers usually include Alt, Control, Shift, Super and similar.
    ///    Effectively, this means that non-significant modifiers are masked out,
    ///    before doing the comparison as described above.
    /// 3. The matching must be independent of the layout/keymap. For example,
    ///    the \<Plus\> (+) symbol is found on the first level on some layouts,
    ///    and requires holding Shift on others. If you simply bind the action
    ///    to the \<Plus\> keysym, it would work for the unshifted kind, but
    ///    not for the others, because the match against Shift would fail. If
    ///    you bind the action to \<Shift\>\<Plus\>, only the shifted kind would
    ///    work. So what is needed is to recognize that Shift is used up in the
    ///    translation of the keysym itself, and therefore should not be included
    ///    in the matching.
    ///    Effectively, this means that consumed modifiers (Shift in this example)
    ///    are masked out as well, before doing the comparison.
    ///
    /// To summarize, this is how the matching would be performed:
    /// @code
    ///   (keysym == shortcut_keysym) &&
    ///   ((state_modifiers & ~consumed_modifiers & significant_modifiers) == shortcut_modifiers)
    /// @endcode
    ///
    /// @c state_modifiers are the modifiers reported by
    /// xkb_state_mod_index_is_active() and similar functions.
    /// @c consumed_modifiers are the modifiers reported by
    /// xkb_state_mod_index_is_consumed().
    /// @c significant_modifiers are decided upon by the application/toolkit/user;
    /// it is up to them to decide whether these are configurable or hard-coded.
    ///
    /// @returns 1 if the modifier is consumed, 0 if it is not.  If the modifier
    /// index is not valid in the keymap, returns -1.
    ///
    /// @sa xkb_state_mod_mask_remove_consumed()
    /// @sa xkb_state_key_get_consumed_mods()
    /// @memberof xkb_state
    pub fn mod_index_is_consumed(&self, key: Keycode, idx: ModIndex)
            -> bool {
        unsafe {
            xkb_state_mod_index_is_consumed(self.ptr, key, idx) != 0
        }
    }


    /// Remove consumed modifiers from a modifier mask for a key.
    ///
    /// Takes the given modifier mask, and removes all modifiers which are
    /// consumed for that particular key (as in xkb_state_mod_index_is_consumed()).
    ///
    /// @sa xkb_state_mod_index_is_consumed()
    /// @memberof xkb_state
    pub fn mod_mask_remove_consumed(&self, key: Keycode, mask: ModMask)
            -> ModMask {
        unsafe {
            xkb_state_mod_mask_remove_consumed(self.ptr, key, mask)
        }
    }


    /// Get the mask of modifiers consumed by translating a given key.
    ///
    /// @returns a mask of the consumed modifiers.
    ///
    /// @sa xkb_state_mod_index_is_consumed()
    /// @memberof xkb_state
    pub fn key_get_consumed_mods(&self, key: Keycode)
            -> ModMask {
        unsafe {
            xkb_state_key_get_consumed_mods(self.ptr, key)
        }
    }


    /// Test whether a layout is active in a given keyboard state by name.
    ///
    /// @returns 1 if the layout is active, 0 if it is not.  If no layout with
    /// this name exists in the keymap, return -1.
    ///
    /// If multiple layouts in the keymap have this name, the one with the lowest
    /// index is tested.
    ///
    /// @sa xkb_layout_index_t
    /// @memberof xkb_state
    pub fn layout_name_is_active<S: AsRef<str>+?Sized>(&self,
                                                       name: &S,
                                                       type_: StateComponent)
            -> bool {
        unsafe {
            let cname = CString::new(name.as_ref().as_bytes()).unwrap();
            xkb_state_layout_name_is_active(self.ptr, cname.as_ptr(), type_) != 0
        }
    }


    /// Test whether a layout is active in a given keyboard state by index.
    ///
    /// @returns 1 if the layout is active, 0 if it is not.  If the layout index
    /// is not valid in the keymap, returns -1.
    ///
    /// @sa xkb_layout_index_t
    /// @memberof xkb_state
    pub fn layout_index_is_active(&self, idx: LayoutIndex,
                                  type_: StateComponent)
            -> bool {
        unsafe {
            xkb_state_layout_index_is_active(self.ptr, idx, type_) != 0
        }
    }


    /// Test whether a LED is active in a given keyboard state by name.
    ///
    /// @returns 1 if the LED is active, 0 if it not.  If no LED with this name
    /// exists in the keymap, returns -1.
    ///
    /// @sa xkb_led_index_t
    /// @memberof xkb_state
    pub fn led_name_is_active<S: AsRef<str>+?Sized>(&self, name: &S)
            -> bool {
        unsafe {
            let cname = CString::new(name.as_ref().as_bytes()).unwrap();
            xkb_state_led_name_is_active(self.ptr, cname.as_ptr()) != 0
        }
    }


    /// Test whether a LED is active in a given keyboard state by index.
    ///
    /// @returns 1 if the LED is active, 0 if it not.  If the LED index is not
    /// valid in the keymap, returns -1.
    ///
    /// @sa xkb_led_index_t
    /// @memberof xkb_state
    pub fn led_index_is_active(&self, idx: LedIndex)
            -> bool {
        unsafe {
            xkb_state_led_index_is_active(self.ptr, idx) != 0
        }
    }
}

impl Clone for State {
    fn clone(&self) -> State {
        unsafe {
            State {
                ptr: xkb_state_ref(self.ptr)
            }
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            xkb_state_unref(self.ptr);
        }
    }
}

