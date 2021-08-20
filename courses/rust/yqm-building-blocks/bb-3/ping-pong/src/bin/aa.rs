fn main() {
    println!("Hello, world!");
    let buf = [1, 2, 3, 4, 5];
    let c = &buf[1..3];
    // let b:[i32; 3] = buf[1..3];
    println!("{:?}", c);
}