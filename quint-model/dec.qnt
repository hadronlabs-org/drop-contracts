// -*- mode: Bluespec; -*-

/**
 * Rational numbers, represented as a tuple of (numerator, denominator)
 * Once this will be a part of Quint's standard library.
 * Andrey Kuprianov, Informal Systems, 2023
 */
module Dec {
    type Dec = (int, int)

    pure def dec(i: int): Dec =
        (i, 1)

    pure def add(a: Dec, b: Dec): Dec =
        (a._1 * b._2 + b._1 * a._2, a._2 * b._2)

    pure def sub(a: Dec, b: Dec): Dec =
        (a._1 * b._2 - b._1 * a._2, a._2 * b._2)

    pure def mul(a: Dec, b: Dec): Dec =
        (a._1 * b._1, a._2 * b._2)

    // assumes that b!=0
    pure def div(a: Dec, b: Dec): Dec =
        (a._1 * b._2, a._2 * b._1)

    pure def pow(a: Dec, n: int): Dec =
        if(a._1 == 0) a
        else
            if(n>=0) (a._1 ^ n, a._2 ^ n)
            else (a._2 ^ (-n), a._1 ^ (-n))

    pure def gt(a: Dec, b: Dec): bool =
        (a.sub(b))._1 > 0 

    pure def equal(a: Dec, b: Dec): bool =
        (a.sub(b))._1 == 0
    
    pure def le(a: Dec, b: Dec): bool =
        (a.sub(b))._1 <= 0

    pure def truncate(a: Dec): int =
        a._1 / a._2

    pure def ceil(a: Dec): int =
        val quo = a._1 / a._2
        if(a._1 % a._2 <= 0) quo
        else quo + 1
}