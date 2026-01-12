mod algo;

fn main() {
    let arr = vec![64, 34, 25, 12, 22, 11, 90];
    let res= algo::sort::bubble_sort(arr);
    dbg!("Sorted array: {:?}", res);
}
