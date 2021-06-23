use byteorder::{LittleEndian, BigEndian};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::Cursor; //extend read/write methods to primitive types

fn write_numbers() -> (u32, i8, f32) {
    let mut w = vec![];

    let one:u32 = 256;
    let two:i8 = 2;
    let three:f32 = 3.0;

    // little endian = least important bytes go first
    // each element in a vector = a byte, not a bit, not a 4-bit (hex letter)
    // hence each element can be 0-255

    // 32-bit number = 4 bytes = adds 4 slots to vector
    w.write_u32::<LittleEndian>(one).unwrap();
    println!("{:?}", &w);

    // 8 bit number = 1 slot
    // interesting that her BigEndian and LittleEndian would look the same, since only one slot
    w.write_i8(two).unwrap();
    println!("{:?}", &w);

    // big endian 64 64 0 0 matches 01000000010000000000000000000000 from https://www.h-schmidt.net/FloatConverter/IEEE754.html for 3.0
    // slight diff in 1st (sign) 0, which I guess is due to encoding scheme
    w.write_f32::<BigEndian>(three).unwrap();
    println!("{:?}", &w);

    (one, two, three)
}

fn read_numbers() -> (u32, i8, f32) {
    let mut r = Cursor::new(vec![1,0,0,0,2,0,0,64,64]);
    let one = r.read_u32::<LittleEndian>().unwrap();
    let two = r.read_i8().unwrap();
    let three = r.read_f32::<LittleEndian>().unwrap();

    println!("{}, {}, {}", one, two, three);

    (one, two, three)
}

fn main() {
    write_numbers();
    read_numbers();
}