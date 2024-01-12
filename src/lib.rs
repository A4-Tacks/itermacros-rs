#![doc = include_str!("../README.md")]

/// Use pattern unpack iterator
///
/// The expression after the equal sign must implement [`IntoIterator`].\
/// The final pattern may be followed by a trailing comma.
///
/// Use else to handle iterator length mismatch or pattern mismatch
///
/// - Use `*name` pattern any elements to [`Vec`],
///   use [`DoubleEndedIterator`] pattern end elements.
/// - Use `*name: Type` pattern any elements collect into impl [`FromIterator`].
/// - Use `*=iter` pattern start and end elements, do not check iter is stopped.
///   Internally use the given variable name to store iterator for future use.
/// - Use `**name` like `*name`, but use [`Iterator`]
///
/// There may be an internal loop, please use the label to break or continue.
///
/// [`FromIterator`]: std::iter::FromIterator
/// [`Iterator`]: std::iter::Iterator
/// [`IntoIterator`]: std::iter::IntoIterator
/// [`DoubleEndedIterator`]: std::iter::DoubleEndedIterator
/// [`Vec`]: std::vec::Vec
///
/// # Examples
///
/// Sized iterator
/// ```
/// # use itermacros::iunpack;
/// assert_eq!(iunpack!(a, b, c, d, e = 0..5 => {
///     (a, b, c, d, e)
/// } else panic!()), (0, 1, 2, 3, 4));
///
/// assert_eq!(iunpack!(a, b, c, d, e = 0..3 => {
///     panic!()
/// } else(err) {
///     err
/// }), 3); // fail, not enough values
///
/// assert_eq!(iunpack!(a, b, c, d, e = 0..7 => {
///     panic!()
/// } else(err) {
///     err
/// }), 5); // fail, too many values
/// ```
///
/// Any size iterator
/// ```
/// # use itermacros::iunpack;
/// # use std::collections::HashSet;
/// assert_eq!(iunpack!(a, b, *c, d, e = 0..8 => {
///     (a, b, c, d, e)
/// } else panic!()), (0, 1, vec![2, 3, 4, 5], 6, 7));
///
/// assert_eq!(iunpack!(a, b, *=c, d, e = 0..8 => {
///     (a, b, c.collect::<Vec<_>>(), d, e)
/// } else panic!()), (0, 1, vec![2, 3, 4, 5], 6, 7));
///
/// assert_eq!(iunpack!(a, b, *c: HashSet<_>, d, e = 0..8 => {
///     (a, b, c, d, e)
/// } else panic!()), (0, 1, HashSet::from([2, 3, 4, 5]), 6, 7));
///
/// // use Iterator, is not DoubleEndedIterator
/// assert_eq!(iunpack!(a, b, **c, d, e = 0..8 => {
///     (a, b, c, d, e)
/// } else panic!()), (0, 1, vec![2, 3, 4, 5], 6, 7));
///
/// // no collect
/// assert_eq!(iunpack!(a, b, *, d, e = 0..8 => {
///     (a, b, d, e)
/// } else panic!()), (0, 1, 6, 7));
///
/// // use Iterator, is not DoubleEndedIterator, no collect
/// assert_eq!(iunpack!(a, b, **, d, e = 0..8 => {
///     (a, b, d, e)
/// } else panic!()), (0, 1, 6, 7));
/// ```
///
/// Pattern example
/// ```
/// # use itermacros::iunpack;
/// # use std::collections::HashSet;
/// assert_eq!(iunpack!(_a, _b, 2..=10 = 0..3 => { true } else panic!()), true);
///
/// assert_eq!(iunpack!((0 | 1 | 2), _b, _c = 0..3 => {
///     true
/// } else panic!()), true);
///
/// assert_eq!(iunpack!(*, 2..=10 = 0..3 => {
///     true
/// } else panic!()), true);
///
/// assert_eq!(iunpack!((0 | 1 | 2), * = 0..3 => {
///     true
/// } else panic!()), true);
///
/// // fails
/// assert_eq!(iunpack!(_a, _b, 3..=10 = 0..3 => {
///     panic!()
/// } else true), true);
///
/// assert_eq!(iunpack!((1 | 2), _b, _c = 0..3 => {
///     panic!()
/// } else true), true);
///
/// assert_eq!(iunpack!(_a, **, 3..=10 = 0..3 => {
///     panic!()
/// } else true), true);
///
/// assert_eq!(iunpack!((1 | 2), * = 0..3 => {
///     panic!()
/// } else true), true);
/// ```
#[macro_export]
macro_rules! iunpack {
    (@if($($t:tt)*) else $($f:tt)*) => ($($t)*);
    (@if else $($f:tt)*) => ($($f)*);
    {@revpat_do_iter_back($iter:ident, $body:block, $errbody:expr)
        $(($used:pat) $(($pat:pat))*)?
    } => {
        $crate::iunpack!(@if$((
            $crate::iunpack!(
                @revpat_do_iter_back($iter, {
                    if let ::core::option::Option::Some($used)
                    = ::core::iter::DoubleEndedIterator::next_back(&mut $iter)
                    {
                        $body
                    } else {
                        $errbody
                    }
                }, $errbody) $(($pat))*
            )
        ))? else $body)
    };
    {@sized_pat($iter:ident, $body:block, $errbody:block, $errval:ident)
        $($fpat:pat $(, $pat:pat)*)?
    } => {
        $crate::iunpack!(@if$((
            if let ::core::option::Option::Some($fpat)
            = ::core::iter::Iterator::next(&mut $iter) {
                $errval += 1;
                $crate::iunpack!(@sized_pat(
                    $iter,
                    $body,
                    $errbody,
                    $errval
                ) $($pat),*)
            } else $errbody
        ))? else $body)
    };
    {@sized_pat($iter:ident, $body:block, $errbody:block)
        $($fpat:pat $(, $pat:pat)*)?
    } => {
        $crate::iunpack!(@if$((
            if let ::core::option::Option::Some($fpat)
            = ::core::iter::Iterator::next(&mut $iter) {
                $crate::iunpack!(@sized_pat(
                    $iter,
                    $body,
                    $errbody
                ) $($pat),*)
            } else $errbody
        ))? else $body)
    };
    // unused err value
    {
        $($pat:pat),* $(,)?
        = $iter:expr => $body:block
        else $errbody:expr
    } => {{
        let mut __iter = ::core::iter::IntoIterator::into_iter($iter);
        $crate::iunpack!(@sized_pat(__iter, {
            if let ::core::option::Option::Some(_)
            = ::core::iter::Iterator::next(&mut __iter) {
                $errbody
            } else {
                $body
            }
        }, { $errbody }) $($pat),*)
    }};
    // used err value
    {
        $($pat:pat),* $(,)?
        = $iter:expr => $body:block
        else($err:ident) $errbody:block
    } => {{
        let mut __iter = ::core::iter::IntoIterator::into_iter($iter);
        let mut $err = 0usize;
        $crate::iunpack!(@sized_pat(__iter, {
            if let ::core::option::Option::Some(_)
            = ::core::iter::Iterator::next(&mut __iter) {
                $errbody
            } else {
                $body
            }
        }, { $errbody }, $err) $($pat),*)
    }};
    // use DoubleEndedIterator
    {
        $($fpat:pat ,)* * $($mid:ident $(: $ty:ty)?)? $(, $bpat:pat)* $(,)?
        = $iter:expr => $body:block
        else $errbody:expr
    } => {{
        let mut __iter = ::core::iter::IntoIterator::into_iter($iter);
        $crate::iunpack!(@sized_pat(__iter, {
            $crate::iunpack!(
                @revpat_do_iter_back(__iter, {
                    $(
                    let $mid = <$crate::iunpack!(
                        @if$(($ty))?else ::std::vec::Vec<_>)
                        as ::core::iter::FromIterator<_>>
                        ::from_iter(__iter);
                    )?
                    $body
                }, $errbody)
                $(($bpat))*
            )
        }, { $errbody }) $($fpat),*)
    }};
    // use DoubleEndedIterator and result mid iterator
    {
        $($fpat:pat ,)* *=$mid:ident $(, $bpat:pat)* $(,)?
        = $iter:expr => $body:block
        else $errbody:expr
    } => {{
        let mut $mid = ::core::iter::IntoIterator::into_iter($iter);
        $crate::iunpack!(@sized_pat($mid, {
            $crate::iunpack!(
                @revpat_do_iter_back($mid, {
                    $body
                }, $errbody)
                $(($bpat))*
            )
        }, { $errbody }) $($fpat),*)
    }};
    // use Iterator unnamed
    {
        $($fpat:pat ,)* ** $(, $bpat:pat)+ $(,)?
        = $iter:expr => $body:block
        else $errbody:expr
    } => {loop {
        let mut __iter = ::core::iter::IntoIterator::into_iter($iter);
        break $crate::iunpack!(@sized_pat(__iter, {
            let mut __buf = [$(
                match ::core::iter::Iterator::next(&mut __iter) {
                    ::core::option::Option::Some(
                        $crate::iunpack!(@if(x) else $bpat)
                    ) => x,
                    ::core::option::Option::None => break $errbody,
                }
            ),+];
            let mut __i = 0;
            while let ::core::option::Option::Some(__elem)
            = ::core::iter::Iterator::next(&mut __iter) {
                __buf[__i] = __elem;
                __i += 1;
                __i %= __buf.len();
            }
            __buf.rotate_left(__i);
            #[allow(irrefutable_let_patterns)]
            if let [$($bpat),+] = __buf {
                $body
            } else { $errbody }
        }, { $errbody }) $($fpat),*)
    }};
    // use Iterator
    {
        $($fpat:pat ,)* ** $mid:ident $(: $ty:ty)? $(, $bpat:pat)+ $(,)?
        = $iter:expr => $body:block
        else $errbody:expr
    } => {loop {
        let mut __iter = ::core::iter::IntoIterator::into_iter($iter);
        break $crate::iunpack!(@sized_pat(__iter, {
            let mut __buf = [$(
                match ::core::iter::Iterator::next(&mut __iter) {
                    ::core::option::Option::Some(
                        $crate::iunpack!(@if(x) else $bpat)
                    ) => x,
                    ::core::option::Option::None => break $errbody,
                }
            ),+];
            let mut __i = 0;
            let mut $mid = <$crate::iunpack!(@if$(($ty))?
                else ::std::vec::Vec<_>)
                as ::core::default::Default>::default();
            while let ::core::option::Option::Some(__elem)
            = ::core::iter::Iterator::next(&mut __iter) {
                ::core::iter::Extend::extend(
                    &mut $mid,
                    ::core::option::Option::Some(
                        ::core::mem::replace(&mut __buf[__i], __elem)
                    )
                );
                __i += 1;
                __i %= __buf.len();
            }
            __buf.rotate_left(__i);
            #[allow(irrefutable_let_patterns)]
            if let [$($bpat),+] = __buf {
                $body
            } else { $errbody }
        }, { $errbody }) $($fpat),*)
    }};
}
