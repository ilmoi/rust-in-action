const BIAS: i32 = 127;
const RADIX: f32 = 2.0;

fn main() {
    let n = 42.42;

    // convert each part into binary
    let (signbit, exponent, fraction) = deconstruct_f32(n);
    println!("{} -> [sign:{:01b}, exponent:{:08b}, mantissa:{:023b}] -> tbc", n, signbit, exponent, fraction);

    // convert each binary part into (modified) decimal
    let (sign, exponent, mantissa) = decode_f32_parts(signbit, exponent, fraction);

    // multiply the 3 together to get the final number
    let reconstituted_n = f32_from_parts(sign, exponent, mantissa);
    println!("{} -> [sign:{}, exponent:{}, mantissa:{:?}] -> {}", n, signbit, exponent, mantissa, reconstituted_n);
}

fn deconstruct_f32(n: f32) -> (u32, u32, u32) {
    // interpret the bit pattern as a u32
    let n_: u32 = unsafe { std::mem::transmute(n) };

    // shift left 31 places to have 1 bit left, then select it with a mask of 1
    let signbit = (n_ >> 31) & 1;

    // shift left 23 places to have 9 bits left, then select 8 of them using a mask of ff
    let exponent = (n_ >> 23) & 0xff;

    // select the last 23 bits
    let fraction = n_ & 0b00000000_01111111_11111111_11111111;

    (signbit, exponent, fraction)
}

fn decode_f32_parts(signbit: u32, exponent: u32, fraction: u32) -> (f32, f32, f32) {
    // if signbit is 0, then the result is 1, else -1
    let sign = (-1.0_f32).powf(signbit as f32);

    // subtract the bias from the exponent, then raise radix to that power
    let exponent = RADIX.powf((exponent as i32 - BIAS) as f32);

    let mut mantissa: f32 = 1.0;
    for i in 0..23_u32 {
        let one_at_bit_i = 1 << i;
        if (one_at_bit_i & fraction) != 0 {
            mantissa += (2.0_f32).powf(i as f32 - 23.0);
        }
    }

    (sign, exponent, mantissa)
}

fn f32_from_parts(sign: f32, exponent: f32, mantissa: f32) -> f32 {
    sign * exponent * mantissa
}