#[macro_export]
macro_rules! repeat_macro_with_argument_without_1 {
    ( $macro:ident, 2 ) => {
        $macro!(A, B);
    };
    ( $macro:ident, 3 ) => {
        $macro!(A, B, C);
        $crate::repeat_macro_with_argument_without_1!($macro, 2);
    };
    ( $macro:ident, 4 ) => {
        $macro!(A, B, C, D);
        $crate::repeat_macro_with_argument_without_1!($macro, 3);
    };
    ( $macro:ident, 5 ) => {
        $macro!(A, B, C, D, E);
        $crate::repeat_macro_with_argument_without_1!($macro, 4);
    };
    ( $macro:ident, 6 ) => {
        $macro!(A, B, C, D, E, F1);
        $crate::repeat_macro_with_argument_without_1!($macro, 5);
    };
    ( $macro:ident, 7 ) => {
        $macro!(A, B, C, D, E, F1, G);
        $crate::repeat_macro_with_argument_without_1!($macro, 6);
    };
    ( $macro:ident, 8 ) => {
        $macro!(A, B, C, D, E, F1, G, H);
        $crate::repeat_macro_with_argument_without_1!($macro, 7);
    };
    ( $macro:ident, 9 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I);
        $crate::repeat_macro_with_argument_without_1!($macro, 8);
    };
    ( $macro:ident, 10 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J);
        $crate::repeat_macro_with_argument_without_1!($macro, 9);
    };
    ( $macro:ident, 11 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K);
        $crate::repeat_macro_with_argument_without_1!($macro, 10);
    };
    ( $macro:ident, 12 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L);
        $crate::repeat_macro_with_argument_without_1!($macro, 11);
    };
    ( $macro:ident, 13 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M);
        $crate::repeat_macro_with_argument_without_1!($macro, 12);
    };
    ( $macro:ident, 14 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N);
        $crate::repeat_macro_with_argument_without_1!($macro, 13);
    };
    ( $macro:ident, 15 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O);
        $crate::repeat_macro_with_argument_without_1!($macro, 14);
    };
    ( $macro:ident, 16 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P);
        $crate::repeat_macro_with_argument_without_1!($macro, 15);
    };
    ( $macro:ident, 17 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q);
        $crate::repeat_macro_with_argument_without_1!($macro, 16);
    };
    ( $macro:ident, 18 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R);
        $crate::repeat_macro_with_argument_without_1!($macro, 17);
    };
    ( $macro:ident, 19 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S);
        $crate::repeat_macro_with_argument_without_1!($macro, 18);
    };
    ( $macro:ident, 20 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
        $crate::repeat_macro_with_argument_without_1!($macro, 19);
    };
    ( $macro:ident, 21 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
        $crate::repeat_macro_with_argument_without_1!($macro, 20);
    };
    ( $macro:ident, 22 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
        $crate::repeat_macro_with_argument_without_1!($macro, 21);
    };
    ( $macro:ident, 23 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
        $crate::repeat_macro_with_argument_without_1!($macro, 22);
    };
    ( $macro:ident, 24 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
        $crate::repeat_macro_with_argument_without_1!($macro, 23);
    };
    ( $macro:ident, 25 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
        $crate::repeat_macro_with_argument_without_1!($macro, 24);
    };
    ( $macro:ident, 26 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
        $crate::repeat_macro_with_argument_without_1!($macro, 25);
    };
    ( $macro:ident, 27 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA);
        $crate::repeat_macro_with_argument_without_1!($macro, 26);
    };
    ( $macro:ident, 28 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB);
        $crate::repeat_macro_with_argument_without_1!($macro, 27);
    };
    ( $macro:ident, 29 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC);
        $crate::repeat_macro_with_argument_without_1!($macro, 28);
    };
    ( $macro:ident, 30 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD);
        $crate::repeat_macro_with_argument_without_1!($macro, 29);
    };
    ( $macro:ident, 31 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD, AE);
        $crate::repeat_macro_with_argument_without_1!($macro, 30);
    };
    ( $macro:ident, 32 ) => {
        $macro!(A, B, C, D, E, F1, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD, AE, AF);
        $crate::repeat_macro_with_argument_without_1!($macro, 31);
    };
}

#[macro_export]
macro_rules! repeat_macro_with_argument {
    ( $macro:ident, $repetition:tt ) => {
        $crate::repeat_macro_with_argument_without_0!($macro, $repetition);
        $macro!();
    };
}


#[macro_export]
macro_rules! repeat_macro_with_argument_without_0 {
    ( $macro:ident, $repetition:tt ) => {
        $crate::repeat_macro_with_argument_without_1!($macro, $repetition);
        $macro!(A);
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
