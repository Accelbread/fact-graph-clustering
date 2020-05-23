//! Helper macros for internal crate use.

/// Macro to automatically implement Deref and DerefMut for newtypes.
///
/// # Examples
///
/// ```
/// newtype_deref!{
///     /// Documentation
///     #[derive(Clone, Debug)]
///     pub struct NewType(pub u32);
///
///     /// Documentation
///     #[derive(Clone, Debug)]
///     pub struct NewType2(pub u32);
/// }
///
/// let x: NewType = NewType(0);
/// let y: u32 = x;
/// ```
macro_rules! newtype_deref {
    (
        $(
            $(#[$attr:meta])*
            $v:vis struct $n:ident$(<$tp:tt>)?($fv:vis $t:ty);
        )*
    ) => {
        $(
            $(#[$attr])*
            $v struct $n$(<$tp>)?($fv $t);

            impl$(<$tp>)? Deref for $n$(<$tp>)? {
                type Target = $t;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl$(<$tp>)? DerefMut for $n$(<$tp>)? {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        )*
    };
}
