# rust 相关的笔记
## 引用和借用
### 不同情况下的&的含义
- 在类型声明上，表示应用类型， &T
```rust
let a:&i32; // 此时&i32是类型声明，表示一个 i32引用类型
```

- 在表达式上，表示的是借用，结果是得到一个应用类型
```
let a = &123i32; // 此时&123 表示是借用，得到一个&i32的引用类型，所以a的类型是&i32
```

- 在变量绑定上，表示解引用，与*类似
```rust
let a = &123;
let &b=a; // 此时b的类型是i32，&b表示解引用，所以b的值是123；等价于 let b = *a
```

### 不同情况下的ref的含义
- 在变量绑定上，表示引用类型
```rust 
let ref a = 123;// 此时表示a的类型是&i32，等价于let a = &123
```

- 在模式匹配上，表示引用类型
```rust
fn main(){
    let s = Some(String::from("Hello"));
    match s{
        Some(ref t) => println!("{}",t), // ref引用类型，此时s的所有权不会转移给t
        _ => {}
    }
    println!("s = {}",s.unwrap());// 依然可以访问s
}

```
## 属性#[]
它是由一个#开启，后面紧接着一个[]，里面便是属性的具体内容，它可以有如下几种写法：  
单个标识符代表的属性名，如#[unix]；    
单个标识符代表属性名，后面紧跟着一个=，然后再跟着一个字面量（Literal），组成一个键值对，如#[link(name = “openssl”)]；    
单个标识符代表属性名，后面跟着一个逗号隔开的子属性的列表，如#[cfg(and(unix, not(windows)))]；   
在#后面还可以紧跟一个!，比如#![feature(box_syntax)]，这表示这个属性是应用于它所在的这个Item。而如果没有!则表示这个属性仅应用于紧接着的那个Item。

前面提到了，在Rust中属性只能用于Item，它有以下七个主要用途：
1、条件编译代码；    
2、设置 crate 名称、版本和类型；   
3、禁用 lint 警告；   
4、启用编译器的特性（如宏、全局导入等）；   
5、连接到一个非 Rust 语言的库；   
6、标记函数作为单元测试；   
7、标记函数作为基准测试的某个部分；  
表现在Rust中可以在模块、crate 或者项中进行应用：   
在整个 crate 应用的语法为：   
```rust
#![crate_attribute] （有感叹号！）
```
在模块或者项应用的语法为：   
```rust
#[item_attribute] （无感叹号）
```

## String 和 &str
"hello" 是个字符串切片， let a = "hello" 的类型是 &str, 用 "hello".to_string() 返回一个String类型.&str不可修改。
### 转换
- &str转String
```
String::from("hello,world")
```
```
"hello,world".to_string()
```
- String转&str
取引用即可:
```rust
fn main() {
    let s = String::from("hello,world!");
    say_hello(&s);
    say_hello(&s[..]);
    say_hello(s.as_str());
}

fn say_hello(s: &str) {
    println!("{}",s);
}
```
实际上这种灵活用法是因为 deref 隐式强制转换，具体我们会在 Deref 特征进行详细讲解。
### String 操作
无法从String取索引,String没有实现所以操作,下面的代码会报错：
```
   let s1 = String::from("hello");
   let h = s1[0];
```
- 追加(push)    
在字符串尾部可以使用 push() 方法追加字符 char，也可以使用 push_str() 方法追加字符串字面量。这两个方法都是在原有的字符串上追加，并不会返回新的字符串。由于字符串追加操作要修改原来的字符串，则该字符串必须是可变的，即字符串变量必须由 mut 关键字修饰。
```
fn main() {
    let mut s = String::from("Hello ");

    s.push_str("rust");
    println!("追加字符串 push_str() -> {}", s);

    s.push('!');
    println!("追加字符 push() -> {}", s);
}
```

- 插入(insert)   

## 结构体
- 结构体的缩略的方法进行初始化
```rust
fn build_user(email: String, username: String) -> User {
    User {
        email: email,
        username: username,
        active: true,
        sign_in_count: 1,
    }
}
```
和字段同名的可以缩略
```
fn build_user(email: String, username: String) -> User {
    User {
        email,
        username,
        active: true,
        sign_in_count: 1,
    }
}
```
- 结构体的更新语法
在实际场景中，有一种情况很常见：根据已有的结构体实例，创建新的结构体实例，例如根据已有的 user1 实例来构建 user2
```rust
  let user2 = User {
        active: user1.active,
        username: user1.username,
        email: String::from("another@example.com"),
        sign_in_count: user1.sign_in_count,
    };
```
可以缩写
```
  let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };
```
因为 user2 仅仅在 email 上与 user1 不同，因此我们只需要对 email 进行赋值，剩下的通过结构体更新语法 ..user1 即可完成。.. 语法表明凡是我们没有显式声明的字段，全部从 user1 中自动获取。需要注意的是 ..user1 必须在结构体的尾部使用。

- 结构体部分不可用
结构体部分成员的所有权被转移了，这个结构体不能用整个的了，但是可以用部分的. 下面的例子里，String类型的所有权会被转移，导致username不可用，从而不能用整个的user1，但是user1.active还是可以用的。
```
let user1 = User {
    email: String::from("someone@example.com"),
    username: String::from("someusername123"),
    active: true,
    sign_in_count: 1,
};
let user2 = User {
    active: user1.active,
    username: user1.username,
    email: String::from("another@example.com"),
    sign_in_count: user1.sign_in_count,
};
println!("{}", user1.active);
// 下面这行会报错
println!("{:?}", user1);
```

- 结构体尽量不要用引用，会有生命周期的问题。在定义结构体的时候，用了引用，编译器会要求定义生命周期。

- 使用 #[derive(Debug)] 来打印结构体的信息
如果没有加这个定义，那么在println！("{}",a)的时候就会报错，说没有实现`std::fmt::Display`
```rust
struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    println!("rect1 is {}", rect1);
}
```
加上#[derive(Debug)]以后，可以用{:?}来打印
```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    println!("rect1 is {:?}", rect1);
}
```
## trait 特征
### 使用特征作为参数
```rust
pub fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}
```
实现了Summary特征的item参数。可以使用任何实现了Summary的类型作为该函数的参数。
上面实际上是一个语法糖。真正的写法是
```rust
pub fn notify<T:Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}
```
### 特征约束
<T:Summary> 是一个特征约束.    
- 多重约束
```rust
pub fn notify<T: Summary + Display>(item: &T) {}
```
语法糖形式:
```
pub fn notify(item: &(impl Summary + Display)) {}
```
- where 约束
当约束很复杂的时候，可以使用where约束
```rust
fn some_function<T, U>(t: &T, u: &U) -> i32
    where T: Display + Clone,
          U: Clone + Debug
{}

```

- 有条件的实现特征    
例如，标准库为任何实现了 Display 特征的类型实现了 ToString 特征：
```rust
impl<T: Display> ToString for T {
    // --snip--
}

```

##  生命周期
'a 生命周期标注lifetime specifier。是指一个标志的生命周期。 比如 x: &'a str 用'a来表示x变量的生命周期。 
'static 就是静态生命周期，生命周期和应用程序一样长。
https://course.rs/advance/lifetime/static.html

'static 在 Rust 中是相当常见的，例如字符串字面值就具有 'static 生命周期
&'static 对于生命周期有着非常强的要求：一个引用必须要活得跟剩下的程序一样久，才能被标注为 &'static

### 声明生命周期
<'a> 声明生命周期

### 生命周期约束
- 在声明的时候约束
<'a:'b> a的生命周期比b长

- 用where约束
```rust
impl<'a> ImportantExcerpt<'a> {
    fn announce_and_return_part<'b>(&'a self, announcement: &'b str) -> &'b str
    where
        'a: 'b,
    {
        println!("Attention please: {}", announcement);
        self.part
    }
}
```

### &'static 一个static的引用
'static 生命周期表示和程序一样长

