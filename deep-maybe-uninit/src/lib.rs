extern crate alloc;
use ::core::mem::MaybeUninit;
pub use ::deep_maybe_uninit_macro::DeepMaybeUninit;

/// # Safety
///
/// `Self` and `<Self as HasDeepMaybeUninit>::AsDeepMaybeUninit`
/// must have the same memory layout, up to fields being
/// the same, or fields being mapped through
/// `<Field as HasDeepMaybeUninit>`::AsDeepMaybeUninit`.
pub unsafe trait HasDeepMaybeUninit: Sized {
    type AsDeepMaybeUninit: IsDeepMaybeUninit<AsDeepInit = Self>;

    fn forget_init_raw(ptr: *const Self) -> *const Self::AsDeepMaybeUninit {
        ptr.cast()
    }
    fn forget_init_raw_mut(ptr: *mut Self) -> *mut Self::AsDeepMaybeUninit {
        ptr.cast()
    }
    fn forget_init(self) -> Self::AsDeepMaybeUninit {
        unsafe { ::core::mem::transmute_copy(&::core::mem::ManuallyDrop::new(self)) }
    }
    fn uninit() -> Self::AsDeepMaybeUninit {
        Self::AsDeepMaybeUninit::uninit()
    }
    fn boxed_uninit() -> Box<Self::AsDeepMaybeUninit> {
        Self::AsDeepMaybeUninit::boxed_uninit()
    }
}

/// # Safety
///
/// Same as `HasDeepMaybeUninit`, except reverse.
/// Also has to be valid when underlaying storage is uninitialized.
pub unsafe trait IsDeepMaybeUninit: Sized {
    type AsDeepInit: HasDeepMaybeUninit<AsDeepMaybeUninit = Self>;

    fn assume_init_raw(ptr: *const Self) -> *const Self::AsDeepInit {
        ptr.cast()
    }
    fn assume_init_raw_mut(ptr: *mut Self) -> *mut Self::AsDeepInit {
        ptr.cast()
    }
    unsafe fn assume_init(self) -> Self::AsDeepInit {
        ::core::mem::transmute_copy(&::core::mem::ManuallyDrop::new(self))
    }
    unsafe fn boxed_assume_init(self: Box<Self>) -> Box<Self::AsDeepInit> {
        Box::from_raw(Box::into_raw(self) as *mut Self::AsDeepInit)
    }
    fn uninit() -> Self {
        unsafe { ::core::mem::transmute_copy(&::core::mem::MaybeUninit::<Self>::uninit()) }
    }
    fn boxed_uninit() -> Box<Self> {
        let layout = ::core::alloc::Layout::new::<Self>();
        if layout.size() == 0 {
            unsafe { Box::<Self>::from_raw(::core::ptr::NonNull::dangling().as_ptr()) }
        } else {
            unsafe {
                let mem = ::alloc::alloc::alloc_zeroed(layout).cast::<Self>();
                Box::from_raw(mem)
            }
        }
    }
}

unsafe trait DeepMaybeUninitBaseCase {}

macro_rules! unsafe_impl_deep_maybe_uninit_base_case {
    ($($t:ty)*) => {
        $(
            unsafe impl DeepMaybeUninitBaseCase for $t {}
        )*
    };
}

unsafe_impl_deep_maybe_uninit_base_case! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
    f32 f64 char bool
}

macro_rules! unsafe_impl_deep_maybe_uninit_generic_base_case {
    ($({$($modifiers:tt)*})*) => {
        $(
            unsafe impl<T: ?Sized> DeepMaybeUninitBaseCase for $($modifiers)* T {}
        )*
    };
}

unsafe_impl_deep_maybe_uninit_generic_base_case! {
    { & } { &mut } { *const } { *mut }
}

unsafe impl<T: DeepMaybeUninitBaseCase> HasDeepMaybeUninit for T {
    type AsDeepMaybeUninit = MaybeUninit<T>;
}
unsafe impl<T: DeepMaybeUninitBaseCase> IsDeepMaybeUninit for MaybeUninit<T> {
    type AsDeepInit = T;
}

unsafe impl<const N: usize, T: HasDeepMaybeUninit> HasDeepMaybeUninit for [T; N] {
    type AsDeepMaybeUninit = [T::AsDeepMaybeUninit; N];
}
unsafe impl<const N: usize, T: IsDeepMaybeUninit> IsDeepMaybeUninit for [T; N] {
    type AsDeepInit = [T::AsDeepInit; N];
}

macro_rules! impl_deep_maybe_uninit_tuple {
    // Stopping criteria (0-ary tuple)
    () => {
        impl_deep_maybe_uninit_tuple!(@impl);
    };
    // Running criteria (n-ary tuple, with n >= 2)
    ($T:ident $( $U:ident )*) => {
        impl_deep_maybe_uninit_tuple!($( $U )*);
        impl_deep_maybe_uninit_tuple!(@impl $T $( $U )*);
    };
    // "Private" internal implementation
    (@impl $( $T:ident )*) => {
        unsafe impl<$($T: HasDeepMaybeUninit),*> HasDeepMaybeUninit for ($($T,)*) {
            type AsDeepMaybeUninit = ($($T::AsDeepMaybeUninit,)*);
        }
        unsafe impl<$($T: IsDeepMaybeUninit),*> IsDeepMaybeUninit for ($($T,)*) {
            type AsDeepInit = ($($T::AsDeepInit,)*);
        }
    };
}

impl_deep_maybe_uninit_tuple!(A B C D E F G H I J K L);
