
macro_rules! repeat_macro_with_argument_without_0 {
    ( $macro:ident, 1 ) => {
        $macro!(A);
    };
    ( $macro:ident, 2 ) => {
        $macro!(A, B);
        repeat_macro_with_argument_without_0!($macro, 1);
    };
    ( $macro:ident, 3 ) => {
        $macro!(A, B, C);
        repeat_macro_with_argument_without_0!($macro, 2);
    };
    ( $macro:ident, 4 ) => {
        $macro!(A, B, C, D);
        repeat_macro_with_argument_without_0!($macro, 3);
    };
    ( $macro:ident, 5 ) => {
        $macro!(A, B, C, D, E);
        repeat_macro_with_argument_without_0!($macro, 4);
    };
    ( $macro:ident, 6 ) => {
        $macro!(A, B, C, D, E, F1);
        repeat_macro_with_argument_without_0!($macro, 5);
    };
    ( $macro:ident, 7 ) => {
        $macro!(A, B, C, D, E, F1, G);
        repeat_macro_with_argument_without_0!($macro, 6);
    };
    ( $macro:ident, 8 ) => {
        $macro!(A, B, C, D, E, F1, G, H);
        repeat_macro_with_argument_without_0!($macro, 7);
    };
    ( $macro:ident, 9 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I);
        repeat_macro_with_argument_without_0!($macro, 8);
    };
    ( $macro:ident, 10 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J);
        repeat_macro_with_argument_without_0!($macro, 9);
    };
    ( $macro:ident, 11 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K);
        repeat_macro_with_argument_without_0!($macro, 10);
    };
    ( $macro:ident, 12 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L);
        repeat_macro_with_argument_without_0!($macro, 11);
    };
    ( $macro:ident, 13 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M);
        repeat_macro_with_argument_without_0!($macro, 12);
    };
    ( $macro:ident, 14 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N);
        repeat_macro_with_argument_without_0!($macro, 13);
    };
    ( $macro:ident, 15 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O);
        repeat_macro_with_argument_without_0!($macro, 14);
    };
    ( $macro:ident, 16 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P);
        repeat_macro_with_argument_without_0!($macro, 15);
    };
}

macro_rules! repeat_macro_with_argument {
    ( $macro:ident, $repetition:tt ) => {
        repeat_macro_with_argument_without_0!($macro, $repetition);
        $macro!();
    };
}

// macro_rules! repeat_macro_with_argument_with_lifetime_without_0 {
//     ( $macro:ident, 1 ) => {
//         $macro!(('a, A));
//     };
//     ( $macro:ident, 2 ) => {
//         $macro!(('a, A), ('b, B));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 1);
//     };
//     ( $macro:ident, 3 ) => {
//         $macro!(('a, A), ('b, B), ('c, C));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 2);
//     };
//     ( $macro:ident, 4 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 3);
//     };
//     ( $macro:ident, 5 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 4);
//     };
//     ( $macro:ident, 6 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 5);
//     };
//     ( $macro:ident, 7 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 6);
//     };
//     ( $macro:ident, 8 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 7);
//     };
//     ( $macro:ident, 9 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 8);
//     };
//     ( $macro:ident, 10 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I), ('j, J));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 9);
//     };
//     ( $macro:ident, 11 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I), ('j, J), ('k, K));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 10);
//     };
//     ( $macro:ident, 12 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I), ('j, J), ('k, K), ('l, L));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 11);
//     };
//     ( $macro:ident, 13 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I), ('j, J), ('k, K), ('l, L), ('m, M));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 12);
//     };
//     ( $macro:ident, 14 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I), ('j, J), ('k, K), ('l, L), ('m, M), ('n, N));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 13);
//     };
//     ( $macro:ident, 15 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I), ('j, J), ('k, K), ('l, L), ('m, M), ('n, N), ('o, O));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 14);
//     };
//     ( $macro:ident, 16 ) => {
//         $macro!(('a, A), ('b, B), ('c, C), ('d, D), ('e, E), ('f, F1), ('g, G), ('h, H), ('i, I), ('j, J), ('k, K), ('l, L), ('m, M), ('n, N), ('o, O), ('p, P));
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, 15);
//     };
// }
//
// macro_rules! repeat_macro_with_argument_with_lifetime {
//     ( $macro:ident, $repetition:tt ) => {
//         repeat_macro_with_argument_with_lifetime_without_0!($macro, $repetition);
//         $macro!();
//     };
// }
