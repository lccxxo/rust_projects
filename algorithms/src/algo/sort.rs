// 冒泡排序
pub fn bubble_sort(arr: Vec<i32>) -> Vec<i32> {
    let mut a = arr.clone();
    let n = a.len();

    // 外层循环控制次数
    for i in 0..n{
        let mut swapped = false;
        
        // 内层循环进行相邻元素比较和交换
        for j in 0..(n-i-1){
            if a[j] > a[j+1] {
                a.swap(j, j+1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }
    
    a
}