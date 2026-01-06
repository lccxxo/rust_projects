// 实现简单的链表


struct Node<T>{
    data: T,
    next: Option<Box<Node<T>>>,
}


struct LinkedList<T>{
    head: Option<Box<Node<T>>>,
    length: usize,
}


impl<T> LinkedList<T>{
    
    // 关联函数
    fn new() -> Self{
        LinkedList{
            head: None,
            length: 0,
        }
    }

    // 在头部插入
    fn push(&mut self, data: T){
        let new_node = Box::new(Node{
            data,
            next: self.head.take(),
        });
        self.head = Some(new_node);
        self.length += 1;
    }

    // 从头部弹出
    fn pop(&mut self) -> Option<T>{
        self.head.take().map(|node| {
            self.head = node.next;
            self.length -= 1;
            node.data
        })
    }

    // 查看头部元素 不弹出
    fn peek(&self) -> Option<&T>{
        self.head.as_ref().map(|node| &node.data)
    }

    // 获取链表长度
    fn len(&self) -> usize{
        self.length
    }

    // 查看链表是否为空
    fn is_empty(&self) -> bool{
        self.head.is_none()
    }

    // 清空链表
    fn clear(&mut self){
        self.head = None;
        self.length = 0;
    }

    // 遍历链表
    fn for_each<F>(&self, mut func: F)
    where
        F: FnMut(&T),
    {
        let mut current = &self.head;
        while let Some(node) = current {
            func(&node.data);
            current = &node.next;
        }
    }
}   

fn main() {
    let mut list = LinkedList::new();
    println!("创建新链表,是否为空：{}", list.is_empty());

    list.push(10);
    list.push(20);
    println!("添加2个元素后长度: {}", list.len());

    println!("头部元素: {:?}", list.peek());

    print!("遍历链表: ");
    list.for_each(|x| print!("{} ", x));
    println!();

    println!("弹出头部元素: {:?}", list.pop());
    println!("弹出头部元素: {:?}", list.pop());
    println!("删除后长度: {}", list.len());

    // 迭代器使用
    list.push(30);
    list.push(40);
    list.push(50);
    for item in list{
        println!("{}",item);
    }
    println!();

    // 从Vec创建链表
    let mut list2 = LinkedList::from(vec![1,2,3,4,5,6]);
    println!("从vec创建的链表长度: {}",list2.len());
}

// 链表迭代器
struct LinkedListIterator<T>{
    list: LinkedList<T>,
}

impl<T> Iterator for LinkedListIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item>{
        self.list.pop()
    }
}


impl<T> IntoIterator for LinkedList<T>{
    type Item = T;
    type IntoIter = LinkedListIterator<T>;

    fn into_iter(self) -> Self::IntoIter{
        LinkedListIterator {list: self}
    }
}

// 从Vec创建链表
impl<T> From<Vec<T>> for LinkedList<T> {
    fn from(vec: Vec<T>) -> Self {
        let mut list = LinkedList::new();
        for item in vec.into_iter().rev() {
            list.push(item);
        }
        list
    }
}

